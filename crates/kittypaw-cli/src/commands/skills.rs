use std::sync::Arc;
use tokio::sync::Mutex;

use kittypaw_store::Store;

use super::helpers::{db_path, require_provider};

pub(crate) fn run_skills_list() {
    let skills = kittypaw_core::skill::load_all_skills();
    match skills {
        Err(e) => {
            eprintln!("Error loading skills: {e}");
            std::process::exit(1);
        }
        Ok(ref list) if list.is_empty() => {
            println!("No skills found. Use 'kittypaw teach' to create one.");
        }
        Ok(list) => {
            println!("Skills:");
            println!(
                "  {:<16} | {:<7} | {:<8} | {:<18} | enabled",
                "name", "version", "trigger", "schedule"
            );
            for (skill, _) in &list {
                let schedule = if skill.trigger.trigger_type == "schedule" {
                    skill
                        .trigger
                        .natural
                        .as_deref()
                        .or(skill.trigger.cron.as_deref())
                        .unwrap_or("—")
                        .to_string()
                } else {
                    "—".to_string()
                };
                let enabled = if skill.enabled { "yes" } else { "no" };
                println!(
                    "  {:<16} | {:<7} | {:<8} | {:<18} | {}",
                    skill.name, skill.version, skill.trigger.trigger_type, schedule, enabled
                );
            }
        }
    }
}

pub(crate) fn run_skills_disable(name: &str) {
    match kittypaw_core::skill::disable_skill(name) {
        Ok(()) => println!("Skill '{name}' disabled."),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_skills_delete(name: &str) {
    match kittypaw_core::skill::delete_skill(name) {
        Ok(()) => println!("Skill '{name}' deleted."),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub(crate) async fn run_skills_explain(name: &str) {
    let config = kittypaw_core::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let provider = require_provider(&config);

    match kittypaw_core::skill::load_skill(name) {
        Ok(Some((skill, js_code))) => {
            let prompt = format!(
                "Explain this JavaScript skill in plain English. What does it do, what permissions does it need, and when does it run?\n\nSkill name: {}\nTrigger: {} {}\nPermissions: {}\n\nCode:\n{}",
                skill.name,
                skill.trigger.trigger_type,
                skill.trigger.cron.as_deref().or(skill.trigger.keyword.as_deref()).unwrap_or(""),
                skill.permissions.primitives.join(", "),
                js_code
            );

            let messages = vec![kittypaw_core::types::LlmMessage {
                role: kittypaw_core::types::Role::User,
                content: prompt,
            }];

            match provider.generate(&messages).await {
                Ok(resp) => println!("{}", resp.content),
                Err(e) => eprintln!("Failed to generate explanation: {e}"),
            }
        }
        Ok(None) => {
            eprintln!("Skill '{name}' not found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub(crate) async fn run_skill_cli(name: &str, dry_run: bool) {
    let config = kittypaw_core::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let db_path = db_path();
    let store = Arc::new(Mutex::new(Store::open(&db_path).unwrap_or_else(|e| {
        eprintln!("Database error: {e}");
        std::process::exit(1);
    })));

    match kittypaw_core::skill::load_skill(name) {
        Ok(Some((skill, js_code))) => {
            let sandbox = kittypaw_sandbox::sandbox::Sandbox::new(config.sandbox.clone());

            let context = serde_json::json!({
                "event_type": "cli",
                "event_text": "",
                "chat_id": "",
                "skill_name": skill.name,
            });
            let wrapped = format!("const ctx = JSON.parse(__context__);\n{js_code}");

            match sandbox.execute(&wrapped, context).await {
                Ok(result) if result.success => {
                    println!("Output: {}", result.output);
                    if !result.skill_calls.is_empty() {
                        if dry_run {
                            println!("\n[dry-run] Skill calls that would execute:");
                            for call in &result.skill_calls {
                                println!("  {}.{}({:?})", call.skill_name, call.method, call.args);
                            }
                        } else {
                            let preresolved = kittypaw_cli::skill_executor::resolve_storage_calls(
                                &result.skill_calls,
                                &*store.lock().await,
                                Some(&skill.name),
                            );
                            let mut checker = kittypaw_core::capability::CapabilityChecker::from_skill_permissions(&skill.permissions);
                            match kittypaw_cli::skill_executor::execute_skill_calls(
                                &result.skill_calls,
                                &config,
                                preresolved,
                                Some(&skill.name),
                                Some(&mut checker),
                                None,
                            )
                            .await
                            {
                                Ok(results) => {
                                    for r in &results {
                                        if r.success {
                                            println!("  {}.{}: OK", r.skill_name, r.method);
                                        } else {
                                            eprintln!(
                                                "  {}.{}: FAILED {:?}",
                                                r.skill_name, r.method, r.error
                                            );
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Skill execution error: {e}"),
                            }
                        }
                    }
                }
                Ok(result) => {
                    eprintln!("Skill failed: {:?}", result.error);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Execution error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Ok(None) => {
            eprintln!("Skill '{name}' not found.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

pub(crate) async fn run_teach_cli(description: &str) {
    let config = kittypaw_core::config::Config::load().unwrap_or_else(|e| {
        eprintln!("Config error: {e}");
        std::process::exit(1);
    });

    let provider = require_provider(&config);

    let sandbox = kittypaw_sandbox::sandbox::Sandbox::new(config.sandbox.clone());

    println!("Generating skill for: {description}...\n");

    loop {
        match kittypaw_cli::teach_loop::handle_teach(
            description,
            "cli",
            &*provider,
            &sandbox,
            &config,
        )
        .await
        {
            Ok(
                ref result @ kittypaw_cli::teach_loop::TeachResult::Generated {
                    ref code,
                    ref dry_run_output,
                    ref skill_name,
                    ref description,
                    ref permissions,
                    ..
                },
            ) => {
                println!("=== Generated Skill: {skill_name} ===\n");
                println!("Description: {description}");
                println!("Permissions: {}", permissions.join(", "));
                println!("\nCode:\n{code}\n");
                println!("Dry-run output: {dry_run_output}\n");

                // Interactive prompt
                eprint!("[a]pprove / [r]eject / re[g]enerate? ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap_or_default();
                let choice = input.trim().to_lowercase();

                match choice.as_str() {
                    "a" | "approve" | "y" | "yes" => {
                        match kittypaw_cli::teach_loop::approve_skill(result) {
                            Ok(()) => {
                                println!("Skill '{skill_name}' saved to .kittypaw/skills/");
                                return;
                            }
                            Err(e) => {
                                eprintln!("Failed to save: {e}");
                                std::process::exit(1);
                            }
                        }
                    }
                    "r" | "reject" | "n" | "no" => {
                        println!("Skill rejected.");
                        return;
                    }
                    "g" | "regenerate" => {
                        println!("\nRegenerating...\n");
                        continue;
                    }
                    _ => {
                        println!("Unknown choice. Skill rejected.");
                        return;
                    }
                }
            }
            Ok(kittypaw_cli::teach_loop::TeachResult::Error(e)) => {
                eprintln!("Teach failed: {e}");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

pub(crate) fn run_skills_import(path: &str) {
    let dir = std::path::Path::new(path);
    if !dir.is_dir() {
        eprintln!("Error: '{path}' is not a directory");
        std::process::exit(1);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading directory: {e}");
            std::process::exit(1);
        }
    };

    let mut toml_files: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".skill.toml") {
                toml_files.push(path);
            }
        }
    }

    if toml_files.is_empty() {
        println!("No .skill.toml files found in '{path}'.");
        return;
    }

    let mut imported = 0u32;
    for toml_path in &toml_files {
        let toml_content = match std::fs::read_to_string(toml_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {e}", toml_path.display());
                continue;
            }
        };

        let skill: kittypaw_core::skill::Skill = match toml::from_str(&toml_content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Invalid TOML in {}: {e}", toml_path.display());
                continue;
            }
        };

        // Derive JS file path from the TOML file name
        let file_stem = toml_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .trim_end_matches(".skill.toml");
        let js_path = toml_path.with_file_name(format!("{file_stem}.js"));

        if !js_path.exists() {
            eprintln!(
                "Warning: No JS file found for skill '{}' (expected {})",
                skill.name,
                js_path.display()
            );
            continue;
        }

        let perms = skill.permissions.primitives.join(", ");
        let trigger_info = match skill.trigger.trigger_type.as_str() {
            "message" => format!(
                "message (keyword: {})",
                skill.trigger.keyword.as_deref().unwrap_or("none")
            ),
            "schedule" => format!(
                "schedule ({})",
                skill
                    .trigger
                    .cron
                    .as_deref()
                    .or(skill.trigger.natural.as_deref())
                    .unwrap_or("none")
            ),
            other => other.to_string(),
        };

        println!(
            "\nSkill: '{}' | Trigger: {} | Permissions: [{}]",
            skill.name, trigger_info, perms
        );
        eprint!(
            "Import skill '{}' with permissions [{}]? (y/n) ",
            skill.name, perms
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or_default();
        let choice = input.trim().to_lowercase();

        if choice != "y" && choice != "yes" {
            println!("Skipped '{}'.", skill.name);
            continue;
        }

        let js_code = match std::fs::read_to_string(&js_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {e}", js_path.display());
                continue;
            }
        };

        match kittypaw_core::skill::save_skill(&skill, &js_code) {
            Ok(()) => {
                println!("Imported '{}'.", skill.name);
                imported += 1;
            }
            Err(e) => {
                eprintln!("Failed to import '{}': {e}", skill.name);
            }
        }
    }

    println!("\nImported {imported} skills.");
}
