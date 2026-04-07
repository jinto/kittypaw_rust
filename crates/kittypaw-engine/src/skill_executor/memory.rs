use kittypaw_core::error::{KittypawError, Result};
use kittypaw_core::memory::MemoryProvider;
use kittypaw_core::types::SkillCall;

/// Strip control characters from memory keys to prevent injection.
fn sanitize_key(key: &str) -> String {
    key.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

pub(super) fn execute_memory(
    call: &SkillCall,
    store: &kittypaw_store::Store,
    profile_name: Option<&str>,
) -> Result<serde_json::Value> {
    match call.method.as_str() {
        "search" => {
            let query = call
                .args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| KittypawError::Skill("Memory.search: query required".into()))?;
            let limit = call.args.get(1).and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let hits = store.memory_search(query, limit)?;
            Ok(serde_json::json!(hits))
        }
        "save" => {
            let raw_key = call
                .args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| KittypawError::Skill("Memory.save: key required".into()))?;
            let key = sanitize_key(raw_key);
            let value = call
                .args
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or_else(|| KittypawError::Skill("Memory.save: value required".into()))?;
            store.set_user_context(&key, value, "memory")?;

            // Write-through: if this key exists in USER.md, update it there too
            let profile = profile_name.unwrap_or("default");
            let existing = kittypaw_core::profile::load_profile(profile);
            let user_keys = kittypaw_core::profile::extract_user_md_keys(&existing.user_md);
            if user_keys.contains(&key) {
                let mut user_md = existing.user_md;
                kittypaw_core::profile::update_user_md_entry(&mut user_md, &key, value);
                let _ = kittypaw_core::profile::save_user_md(profile, &user_md);
            }

            Ok(serde_json::json!({"saved": true, "key": key}))
        }
        "recall" => {
            let query = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            if query.is_empty() {
                // Return all memory entries
                let entries = store.list_shared_context()?;
                Ok(serde_json::json!(entries))
            } else {
                // Prefix search
                let entries = store.list_user_context_prefix(query)?;
                let map: serde_json::Map<String, serde_json::Value> = entries
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                Ok(serde_json::Value::Object(map))
            }
        }
        "user" => {
            // Update USER.md with a key-value pair
            let raw_key = call
                .args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| KittypawError::Skill("Memory.user: key required".into()))?;
            let key = sanitize_key(raw_key);
            let value = call
                .args
                .get(1)
                .and_then(|v| v.as_str())
                .ok_or_else(|| KittypawError::Skill("Memory.user: value required".into()))?;

            let profile = profile_name.unwrap_or("default");
            let existing = kittypaw_core::profile::load_profile(profile);
            let mut user_md = existing.user_md;

            kittypaw_core::profile::update_user_md_entry(&mut user_md, &key, value);
            kittypaw_core::profile::save_user_md(profile, &user_md)
                .map_err(|e| KittypawError::Skill(format!("Failed to save USER.md: {e}")))?;

            // Write-through: also persist to DB so Memory.recall() can access it
            store.set_user_context(&key, value, "user_profile")?;

            Ok(serde_json::json!({"saved": true, "key": key, "profile": profile}))
        }
        _ => Err(KittypawError::CapabilityDenied(format!(
            "Unknown Memory method: {}",
            call.method
        ))),
    }
}
