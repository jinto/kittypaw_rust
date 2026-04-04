use kittypaw_core::config::Config;
use kittypaw_core::error::{KittypawError, Result};
use kittypaw_core::types::SkillCall;

use super::resolve_channel_token;

/// Extract and validate chat_id (args[0]) and a second required arg (args[1]).
/// Returns the two values or an error naming the missing field.
pub(super) fn require_telegram_args<'a>(
    call: &'a SkillCall,
    second_name: &str,
) -> Result<(&'a str, &'a str)> {
    let chat_id = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
    let second = call.args.get(1).and_then(|v| v.as_str()).unwrap_or("");
    if chat_id.is_empty() {
        return Err(KittypawError::Skill("Telegram: missing chat_id".into()));
    }
    if second.is_empty() {
        return Err(KittypawError::Skill(format!(
            "Telegram: missing {second_name}"
        )));
    }
    Ok((chat_id, second))
}

pub(super) async fn execute_telegram(
    call: &SkillCall,
    config: &Config,
) -> Result<serde_json::Value> {
    // Token resolution chain (token is NOT passed via args — the JS ABI is
    // Telegram.sendMessage(chatId, text), so args carry only chat content):
    // 1. global channel secret from Settings
    // 2. environment variable fallback
    // 3. config.channels[*] where channel_type == "telegram"
    let bot_token = resolve_channel_token(
        config,
        "telegram",
        "telegram_token",
        "KITTYPAW_TELEGRAM_TOKEN",
    )
    .ok_or_else(|| KittypawError::Config("Telegram bot token not configured".into()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| KittypawError::Skill(format!("Telegram client build error: {e}")))?;

    match call.method.as_str() {
        "sendMessage" => {
            // ABI: Telegram.sendMessage(chatId, text)
            let (chat_id, text) = require_telegram_args(call, "text")?;

            let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
            let resp = client
                .post(&url)
                .json(&serde_json::json!({
                    "chat_id": chat_id,
                    "text": text,
                }))
                .send()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram API error: {e}")))?;

            let status = resp.status();
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram response parse error: {e}")))?;

            if !status.is_success() {
                let err = body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(KittypawError::Skill(format!(
                    "Telegram sendMessage error {status}: {err}"
                )));
            }
            Ok(body)
        }
        "sendPhoto" => {
            // ABI: Telegram.sendPhoto(chatId, photoUrl)
            let (chat_id, photo_url) = require_telegram_args(call, "photo_url")?;

            let url = format!("https://api.telegram.org/bot{bot_token}/sendPhoto");
            let resp = client
                .post(&url)
                .json(&serde_json::json!({
                    "chat_id": chat_id,
                    "photo": photo_url,
                }))
                .send()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram API error: {e}")))?;

            let status = resp.status();
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram response parse error: {e}")))?;
            if !status.is_success() {
                let err = body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(KittypawError::Skill(format!(
                    "Telegram sendPhoto error {status}: {err}"
                )));
            }
            Ok(body)
        }
        "sendDocument" => {
            // ABI: Telegram.sendDocument(chatId, fileUrl, caption?)
            let (chat_id, file_url) = require_telegram_args(call, "file_url")?;
            let caption = call.args.get(2).and_then(|v| v.as_str()).unwrap_or("");

            let url = format!("https://api.telegram.org/bot{bot_token}/sendDocument");
            let mut payload = serde_json::json!({
                "chat_id": chat_id,
                "document": file_url,
            });
            if !caption.is_empty() {
                payload["caption"] = serde_json::Value::String(caption.to_string());
            }
            let resp = client
                .post(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram API error: {e}")))?;

            let status = resp.status();
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| KittypawError::Skill(format!("Telegram response parse error: {e}")))?;
            if !status.is_success() {
                let err = body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                return Err(KittypawError::Skill(format!(
                    "Telegram sendDocument error {status}: {err}"
                )));
            }
            Ok(body)
        }
        _ => Err(KittypawError::CapabilityDenied(format!(
            "Unknown Telegram method: {}",
            call.method
        ))),
    }
}
