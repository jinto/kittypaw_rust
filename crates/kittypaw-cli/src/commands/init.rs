pub(crate) fn run_init() {
    let config_path = std::path::Path::new("kittypaw.toml");

    if config_path.exists() {
        eprint!("kittypaw.toml already exists. Overwrite? (y/n) ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_default();
        let choice = input.trim().to_lowercase();
        if choice != "y" && choice != "yes" {
            println!("Aborted.");
            return;
        }
    }

    // Prompt for API key
    eprint!("Enter your Claude API key (sk-ant-...): ");
    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key).unwrap_or_default();
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        eprintln!("Warning: No API key provided. Set KITTYPAW_API_KEY env var before running.");
    }

    // Prompt for Telegram token
    eprint!("Enter Telegram bot token (optional, press Enter to skip): ");
    let mut telegram_token = String::new();
    std::io::stdin()
        .read_line(&mut telegram_token)
        .unwrap_or_default();
    let telegram_token = telegram_token.trim().to_string();

    // Build config content
    let mut content = format!(
        r#"[llm]
provider = "claude"
api_key = "{api_key}"
model = "claude-sonnet-4-20250514"
max_tokens = 4096

[sandbox]
timeout_secs = 30
memory_limit_mb = 64

# Teach settings
admin_chat_ids = []
freeform_fallback = false
"#
    );

    if !telegram_token.is_empty() {
        content.push_str(&format!(
            r#"
[[channels]]
channel_type = "telegram"
token = "{telegram_token}"
"#
        ));
    }

    if let Err(e) = std::fs::write(config_path, &content) {
        eprintln!("Failed to write kittypaw.toml: {e}");
        std::process::exit(1);
    }

    // Create skills directory
    let skills_dir = std::path::Path::new(".kittypaw/skills");
    if let Err(e) = std::fs::create_dir_all(skills_dir) {
        eprintln!("Failed to create .kittypaw/skills/: {e}");
        std::process::exit(1);
    }

    println!(
        r#"
KittyPaw initialized!

Next steps:
  kittypaw teach "send me a daily joke"    # Teach a new skill
  kittypaw serve                            # Start the bot server
  kittypaw skills list                      # View taught skills"#
    );
}

pub(crate) fn run_config_check() {
    match kittypaw_core::config::Config::load() {
        Ok(config) => {
            println!("Config OK");
            println!("  LLM provider : {}", config.llm.provider);
            println!("  LLM model    : {}", config.llm.model);
            println!("  Max tokens   : {}", config.llm.max_tokens);
            println!(
                "  API key      : {}",
                if config.llm.api_key.is_empty() {
                    "NOT SET"
                } else {
                    "set"
                }
            );
            println!("  Sandbox timeout : {}s", config.sandbox.timeout_secs);
            println!("  Sandbox memory  : {}MB", config.sandbox.memory_limit_mb);
            println!("  Channels : {}", config.channels.len());
            for ch in &config.channels {
                let addr = ch.bind_addr.as_deref().unwrap_or("-");
                println!(
                    "    - {} (bind={}, token={})",
                    ch.channel_type,
                    addr,
                    if ch.token.is_empty() {
                        "not set"
                    } else {
                        "set"
                    }
                );
            }
            println!("  Agents   : {}", config.agents.len());
            for agent in &config.agents {
                println!("    - {} ({})", agent.name, agent.id);
            }
            // Check if any LLM provider is available (legacy or [[models]])
            let has_provider = !config.llm.api_key.is_empty()
                || config.models.iter().any(|m| {
                    matches!(m.provider.as_str(), "ollama" | "local") || !m.api_key.is_empty()
                });
            if !has_provider {
                eprintln!(
                    "Warning: No LLM provider configured. Set KITTYPAW_API_KEY, add llm.api_key, or add [[models]] to kittypaw.toml"
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Config error: {e}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_agent_list() {
    let config = kittypaw_core::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    if config.agents.is_empty() {
        println!("No agents configured. Add [[agents]] sections to kittypaw.toml");
        return;
    }

    for agent in &config.agents {
        println!("Agent: {} (id={})", agent.name, agent.id);
        println!(
            "  System prompt: {}...",
            &agent.system_prompt.chars().take(60).collect::<String>()
        );
        println!(
            "  Channels: {}",
            if agent.channels.is_empty() {
                "none".to_string()
            } else {
                agent.channels.join(", ")
            }
        );
        if agent.allowed_skills.is_empty() {
            println!("  Skills: none");
        } else {
            println!("  Skills:");
            for skill in &agent.allowed_skills {
                let methods = if skill.methods.is_empty() {
                    "all".to_string()
                } else {
                    skill.methods.join(", ")
                };
                println!(
                    "    - {} [{}] (rate: {}/min)",
                    skill.skill, methods, skill.rate_limit_per_minute
                );
            }
        }
    }
}
