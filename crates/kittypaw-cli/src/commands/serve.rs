use std::sync::Arc;
use tokio::sync::Mutex;

use kittypaw_channels::channel::Channel;
use kittypaw_channels::slack::SlackChannel;
use kittypaw_channels::telegram::TelegramChannel;
use kittypaw_channels::websocket::ServeWebSocketChannel;
use kittypaw_core::config::ChannelType;
use kittypaw_core::types::EventType;
use kittypaw_store::Store;

use super::helpers::{db_path, require_provider};

/// Routes a response message to the correct channel based on EventType.
struct ResponseRouter<'a> {
    ws_channel: &'a ServeWebSocketChannel,
    config: &'a kittypaw_core::config::Config,
}

impl<'a> ResponseRouter<'a> {
    async fn send_response(&self, event_type: &EventType, session_id: &str, text: &str) {
        match event_type {
            EventType::WebChat => {
                if let Err(e) = self.ws_channel.send_to_session(session_id, text).await {
                    tracing::warn!("Failed to send WebSocket response: {e}");
                }
            }
            EventType::Telegram => {
                send_telegram_message(self.config, session_id, text).await;
            }
            EventType::Desktop => {
                tracing::info!("Desktop response for {session_id}: {text}");
            }
        }
    }

    async fn send_error(&self, event_type: &EventType, session_id: &str, text: &str) {
        match event_type {
            EventType::WebChat => {
                let _ = self.ws_channel.send_to_session(session_id, text).await;
            }
            EventType::Telegram => {
                send_telegram_message(self.config, session_id, text).await;
            }
            EventType::Desktop => {
                tracing::error!("Desktop error for {session_id}: {text}");
            }
        }
    }
}

pub(crate) async fn run_serve(bind_addr: &str) {
    let config = kittypaw_core::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let provider = require_provider(&config);

    let sandbox = kittypaw_sandbox::sandbox::Sandbox::new(config.sandbox.clone());

    let db_path = db_path();
    let store = Arc::new(Mutex::new(Store::open(&db_path).unwrap_or_else(|e| {
        eprintln!("Database error: {e}");
        std::process::exit(1);
    })));

    // Bounded mpsc channel for all incoming events
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<kittypaw_core::types::Event>(256);

    // Start WebSocket channel
    let ws_channel = ServeWebSocketChannel::new(bind_addr);
    let _ws_handle = ws_channel
        .spawn(event_tx.clone())
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to start WebSocket channel: {e}");
            std::process::exit(1);
        });

    // Start Telegram channel if configured
    let telegram_token = std::env::var("KITTYPAW_TELEGRAM_TOKEN")
        .ok()
        .or_else(|| {
            config
                .channels
                .iter()
                .find(|c| c.channel_type == ChannelType::Telegram)
                .map(|c| c.token.clone())
        })
        .unwrap_or_default();
    if !telegram_token.is_empty() {
        let tg_channel = TelegramChannel::new(&telegram_token);
        let tg_tx = event_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tg_channel.start(tg_tx).await {
                tracing::error!("Telegram channel error: {e}");
            }
        });
        eprintln!("Telegram bot polling started.");
    }

    // Start Slack channel if configured (Socket Mode)
    let slack_bot_token = std::env::var("KITTYPAW_SLACK_BOT_TOKEN")
        .ok()
        .or_else(|| {
            config
                .channels
                .iter()
                .find(|c| c.channel_type == ChannelType::Slack)
                .map(|c| c.token.clone())
        })
        .unwrap_or_default();
    let slack_app_token = std::env::var("KITTYPAW_SLACK_APP_TOKEN").unwrap_or_default();
    if !slack_bot_token.is_empty() && !slack_app_token.is_empty() {
        let slack_channel = SlackChannel::new(&slack_bot_token, &slack_app_token);
        let slack_tx = event_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = slack_channel.start(slack_tx).await {
                tracing::error!("Slack channel error: {e}");
            }
        });
        eprintln!("Slack Socket Mode started.");
    }

    eprintln!(
        "kittypaw serve started. WebSocket at ws://{}/ws/chat",
        bind_addr
    );
    eprintln!("Press Ctrl+C to stop.");

    // Spawn schedule evaluator
    let schedule_config = config.clone();
    let schedule_sandbox = kittypaw_sandbox::sandbox::Sandbox::new(config.sandbox.clone());
    let db_path_sched = db_path.clone();
    tokio::spawn(async move {
        kittypaw_cli::schedule::run_schedule_loop(
            &schedule_config,
            &schedule_sandbox,
            &db_path_sched,
        )
        .await;
    });

    // Graceful shutdown signal
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutting down...");
        let _ = shutdown_tx.send(true);
    });

    // Event processing loop
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                tracing::info!("Shutdown signal received, exiting event loop.");
                break;
            }
            maybe_event = event_rx.recv() => {
                let event = match maybe_event {
                    Some(e) => e,
                    None => break,
                };
                // Capture session_id before moving event
                let session_id = match event.event_type {
                    EventType::WebChat => event
                        .payload
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default")
                        .to_string(),
                    EventType::Telegram => event
                        .payload
                        .get("chat_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default")
                        .to_string(),
                    EventType::Desktop => event
                        .payload
                        .get("workspace_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default")
                        .to_string(),
                };
                let event_type = event.event_type.clone();
                let router = ResponseRouter { ws_channel: &ws_channel, config: &config };

                // Check for /teach command on Telegram
                let is_teach = event.event_type == EventType::Telegram
                    && event
                        .payload
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|t| t.starts_with("/teach"))
                        .unwrap_or(false);

                // Extract raw event text for skill matching
                let raw_event_text = event
                    .payload
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if is_teach {
                    let teach_text = raw_event_text.strip_prefix("/teach").unwrap_or("").trim();
                    let chat_id_str = event
                        .payload
                        .get("chat_id")
                        .map(|v| {
                            v.as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| v.to_string())
                        })
                        .unwrap_or_default();

                    if teach_text.is_empty() {
                        send_telegram_message(&config, &chat_id_str, "Usage: /teach <description>\n\nExample: /teach send me a daily joke").await;
                    } else {
                        send_telegram_message(&config, &chat_id_str, &format!("Generating skill for: {teach_text}...")).await;
                        match kittypaw_cli::teach_loop::handle_teach(teach_text, &chat_id_str, &*provider, &sandbox, &config).await {
                            Ok(ref result @ kittypaw_cli::teach_loop::TeachResult::Generated { ref code, ref dry_run_output, ref skill_name, .. }) => {
                                match kittypaw_cli::teach_loop::approve_skill(result) {
                                    Ok(()) => {
                                        let msg = format!(
                                            "Skill '{skill_name}' generated and saved!\n\nCode:\n{code}\n\nDry-run output: {dry_run_output}"
                                        );
                                        send_telegram_message(&config, &chat_id_str, &msg).await;
                                    }
                                    Err(e) => {
                                        send_telegram_message(&config, &chat_id_str, &format!("Failed to save skill: {e}")).await;
                                    }
                                }
                            }
                            Ok(kittypaw_cli::teach_loop::TeachResult::Error(e)) => {
                                send_telegram_message(&config, &chat_id_str, &format!("Teach failed: {e}")).await;
                            }
                            Err(e) => {
                                send_telegram_message(&config, &chat_id_str, &format!("Error: {e}")).await;
                            }
                        }
                    }
                    continue;
                }

                // Check taught skills before falling through to agent loop
                let skills = kittypaw_core::skill::load_all_skills();
                let matched_skill = match skills {
                    Ok(ref skill_list) => skill_list.iter().find(|(skill, _js)| {
                        skill.enabled && kittypaw_core::skill::match_trigger(skill, &raw_event_text)
                    }),
                    Err(ref e) => {
                        tracing::warn!("Failed to load skills: {e}");
                        None
                    }
                };

                if let Some((skill, js_code)) = matched_skill {
                    let wrapped_code = format!("const ctx = JSON.parse(__context__);\n{}", js_code);
                    let context = serde_json::json!({
                        "event_type": format!("{:?}", event_type).to_lowercase(),
                        "event_text": raw_event_text,
                        "chat_id": session_id,
                    });

                    match sandbox.execute(&wrapped_code, context).await {
                        Ok(exec_result) => {
                            if !exec_result.skill_calls.is_empty() {
                                let preresolved = kittypaw_cli::skill_executor::resolve_storage_calls(&exec_result.skill_calls, &*store.lock().await, Some(&skill.name));
                                let mut checker = kittypaw_core::capability::CapabilityChecker::from_skill_permissions(&skill.permissions);
                                let _ = kittypaw_cli::skill_executor::execute_skill_calls(&exec_result.skill_calls, &config, preresolved, Some(&skill.name), Some(&mut checker), None).await;
                            }
                            let output = if exec_result.output.is_empty() {
                                "(no output)".to_string()
                            } else {
                                exec_result.output.clone()
                            };
                            router.send_response(&event_type, &session_id, &output).await;
                        }
                        Err(e) => {
                            tracing::error!("Skill execution error for session {session_id}: {e}");
                            router.send_error(&event_type, &session_id, &format!("Skill error: {e}")).await;
                        }
                    }
                    continue;
                }

                // No skill matched — check freeform fallback
                if !config.freeform_fallback {
                    let msg = "No matching skill found. Use /teach to create one.";
                    router.send_response(&event_type, &session_id, msg).await;
                    continue;
                }

                let assistant_ctx = kittypaw_cli::assistant::AssistantContext {
                    event: &event,
                    provider: &*provider,
                    store: Arc::clone(&store),
                    registry_entries: &[],
                    sandbox: &sandbox,
                    config: &config,
                    on_token: None,
                };
                match kittypaw_cli::assistant::run_assistant_turn(&assistant_ctx).await {
                    Ok(turn) => {
                        router.send_response(&event_type, &session_id, &turn.response_text).await;
                    }
                    Err(e) => {
                        tracing::error!("Assistant error for session {session_id}: {e}");
                        router.send_error(&event_type, &session_id, &format!("Error: {e}")).await;
                    }
                }
            }
        }
    }
}

pub(crate) async fn send_telegram_message(
    config: &kittypaw_core::config::Config,
    chat_id: &str,
    text: &str,
) {
    let bot_token = match std::env::var("KITTYPAW_TELEGRAM_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            // Try channel config
            config
                .channels
                .iter()
                .find(|c| c.channel_type == ChannelType::Telegram)
                .map(|c| c.token.clone())
                .unwrap_or_default()
        }
    };
    if bot_token.is_empty() {
        tracing::warn!("Cannot send Telegram message: no bot token configured");
        return;
    }
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        }))
        .send()
        .await;
    if let Err(e) = res {
        tracing::warn!("Failed to send Telegram message: {e}");
    }
}
