use edge_tts_rust::{EdgeTtsClient, SpeakOptions};
use kittypaw_core::error::{KittypawError, Result};
use kittypaw_core::types::SkillCall;

const DEFAULT_VOICE: &str = "ko-KR-SunHiNeural";

pub(super) async fn execute_tts(call: &SkillCall) -> Result<serde_json::Value> {
    match call.method.as_str() {
        "speak" => {
            let text = call.args.first().and_then(|v| v.as_str()).unwrap_or("");
            if text.is_empty() {
                return Err(KittypawError::Skill("Tts.speak: text is required".into()));
            }

            // Optional second arg: { voice, rate, pitch } or just a voice string
            let voice = call
                .args
                .get(1)
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.get("voice").and_then(|v| v.as_str()).map(String::from))
                })
                .unwrap_or_else(|| DEFAULT_VOICE.to_string());

            let rate = call
                .args
                .get(1)
                .and_then(|v| v.get("rate"))
                .and_then(|v| v.as_str())
                .unwrap_or("+0%")
                .to_string();

            let pitch = call
                .args
                .get(1)
                .and_then(|v| v.get("pitch"))
                .and_then(|v| v.as_str())
                .unwrap_or("+0Hz")
                .to_string();

            let client = EdgeTtsClient::new()
                .map_err(|e| KittypawError::Skill(format!("TTS client error: {e}")))?;

            let result = client
                .synthesize(
                    text,
                    SpeakOptions {
                        voice,
                        rate,
                        pitch,
                        ..SpeakOptions::default()
                    },
                )
                .await
                .map_err(|e| KittypawError::Skill(format!("TTS synthesis error: {e}")))?;

            // Write to temp file
            let tts_dir = std::env::temp_dir().join("kittypaw-tts");
            std::fs::create_dir_all(&tts_dir)?;
            let filename = format!("{}.mp3", uuid_short());
            let path = tts_dir.join(&filename);
            std::fs::write(&path, &result.audio)?;

            Ok(serde_json::json!({
                "path": path.to_string_lossy(),
                "size": result.audio.len(),
            }))
        }
        _ => Err(KittypawError::CapabilityDenied(format!(
            "Unknown Tts method: {}",
            call.method
        ))),
    }
}

fn uuid_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", t.as_secs(), t.subsec_nanos())
}
