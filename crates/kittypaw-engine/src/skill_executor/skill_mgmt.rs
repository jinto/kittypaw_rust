use kittypaw_core::config::Config;
use kittypaw_core::error::{KittypawError, Result};
use kittypaw_core::skill::{Skill, SkillFormat, SkillPermissions, SkillTrigger};
use kittypaw_core::types::SkillCall;

pub(super) async fn execute_skill_mgmt(
    call: &SkillCall,
    config: &Config,
) -> Result<serde_json::Value> {
    match call.method.as_str() {
        "create" => {
            let name = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            let description = call.args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            let code = call.args.get(2).and_then(|v| v.as_str()).unwrap_or("");
            let trigger_type = call
                .args
                .get(3)
                .and_then(|v| v.as_str())
                .unwrap_or("message");
            let trigger_value = call.args.get(4).and_then(|v| v.as_str()).unwrap_or("");

            if name.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.create: name is required".into(),
                ));
            }
            if code.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.create: code is required".into(),
                ));
            }

            let trigger = if trigger_type == "schedule" {
                if trigger_value.is_empty() {
                    return Err(KittypawError::Sandbox(
                        "Skill.create: schedule trigger requires a schedule expression as the 5th argument (e.g. \"every 10m\" or \"*/10 * * * *\")".into(),
                    ));
                }
                // parse_schedule handles "every 10m", 5-field cron → 6-field conversion
                let cron_expr = crate::teach_loop::parse_schedule(trigger_value)?;
                SkillTrigger {
                    trigger_type: "schedule".into(),
                    cron: Some(cron_expr),
                    natural: Some(description.to_string()),
                    keyword: None,
                    run_at: None,
                }
            } else if trigger_type == "once" {
                if trigger_value.is_empty() {
                    return Err(KittypawError::Sandbox(
                        "Skill.create: once trigger requires a delay as the 5th argument (e.g. \"2m\", \"10m\", \"1h\")".into(),
                    ));
                }
                let run_at = crate::teach_loop::parse_once_delay(trigger_value)?;
                SkillTrigger {
                    trigger_type: "once".into(),
                    cron: None,
                    natural: Some(trigger_value.to_string()),
                    keyword: None,
                    run_at: Some(run_at.to_rfc3339()),
                }
            } else {
                SkillTrigger {
                    trigger_type: "message".into(),
                    cron: None,
                    natural: None,
                    keyword: if trigger_value.is_empty() {
                        Some(name.to_string())
                    } else {
                        Some(trigger_value.to_string())
                    },
                    run_at: None,
                }
            };

            // Detect permissions from code
            let mut perms = Vec::new();
            for prim in [
                "Http", "Web", "Telegram", "Slack", "Discord", "Storage", "Llm", "Shell", "Git",
                "File", "Tts", "Memory", "Todo", "Skill", "Agent", "Moa", "Image", "Vision", "Env",
            ] {
                if code.contains(prim) {
                    perms.push(prim.to_string());
                }
            }

            let now = chrono::Utc::now().to_rfc3339();
            let skill = Skill {
                name: name.to_string(),
                version: 1,
                description: description.to_string(),
                created_at: now.clone(),
                updated_at: now,
                enabled: true,
                trigger,
                permissions: SkillPermissions {
                    primitives: perms,
                    allowed_hosts: vec![],
                },
                format: SkillFormat::Native,
                model_tier: None,
            };

            kittypaw_core::skill::save_skill(&skill, code)?;
            tracing::info!(name = name, "Skill created by LLM");
            Ok(serde_json::json!({"ok": true, "name": name, "description": description}))
        }
        "list" => {
            let skills = kittypaw_core::skill::load_all_skills()?;
            let list: Vec<_> = skills
                .iter()
                .map(|(s, _)| {
                    serde_json::json!({
                        "name": s.name,
                        "description": s.description,
                        "enabled": s.enabled,
                        "triggerType": s.trigger.trigger_type,
                        "triggerValue": s.trigger.natural.as_deref()
                            .or(s.trigger.keyword.as_deref())
                            .unwrap_or(""),
                    })
                })
                .collect();
            // Return array directly — LLM expects skills.length, not skills.skills.length
            Ok(serde_json::json!(list))
        }
        "delete" => {
            let name = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.delete: name is required".into(),
                ));
            }
            // Archive before delete (version increment)
            let _ = kittypaw_core::skill::version_increment(name);
            tracing::info!(name = name, "Skill deleted by LLM");
            Ok(serde_json::json!({"ok": true}))
        }
        "update" => {
            let name = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            let modification = call.args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.update: name is required".into(),
                ));
            }
            if modification.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.update: modification description is required".into(),
                ));
            }
            let (_, existing_code) = kittypaw_core::skill::load_skill(name)?.ok_or_else(|| {
                KittypawError::Sandbox(format!("Skill.update: skill '{name}' not found"))
            })?;

            let registry = build_llm_registry(config);
            let provider = registry.default_provider().ok_or_else(|| {
                KittypawError::Sandbox("Skill.update: no LLM provider configured".into())
            })?;

            crate::teach_loop::handle_modify(name, modification, &existing_code, provider.as_ref())
                .await?;
            tracing::info!(
                name = name,
                modification = modification,
                "Skill updated by LLM"
            );
            Ok(serde_json::json!({"ok": true, "name": name}))
        }
        "rollback" => {
            let name = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                return Err(KittypawError::Sandbox(
                    "Skill.rollback: name is required".into(),
                ));
            }
            kittypaw_core::skill::rollback_skill(name)?;
            tracing::info!(name = name, "Skill rolled back by LLM");
            Ok(serde_json::json!({"ok": true, "name": name}))
        }
        _ => Err(KittypawError::Sandbox(format!(
            "Unknown Skill method: {}",
            call.method
        ))),
    }
}

/// Build an LlmRegistry from config — handles both [[models]] and legacy [llm] sections.
fn build_llm_registry(config: &Config) -> kittypaw_llm::registry::LlmRegistry {
    if !config.models.is_empty() {
        let mut models = config.models.clone();
        if !config.llm.api_key.is_empty() {
            for model in &mut models {
                if model.api_key.is_empty()
                    && matches!(model.provider.as_str(), "claude" | "anthropic" | "openai")
                {
                    model.api_key = config.llm.api_key.clone();
                }
            }
        }
        kittypaw_llm::registry::LlmRegistry::from_configs(&models)
    } else if !config.llm.api_key.is_empty() {
        let legacy = kittypaw_core::config::ModelConfig {
            name: config.llm.provider.clone(),
            provider: config.llm.provider.clone(),
            model: config.llm.model.clone(),
            api_key: config.llm.api_key.clone(),
            max_tokens: config.llm.max_tokens,
            default: true,
            base_url: None,
            context_window: None,
            tier: None,
        };
        kittypaw_llm::registry::LlmRegistry::from_configs(&[legacy])
    } else {
        kittypaw_llm::registry::LlmRegistry::new()
    }
}
