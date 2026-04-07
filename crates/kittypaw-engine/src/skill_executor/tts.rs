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
            let raw_voice = call
                .args
                .get(1)
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.get("voice").and_then(|v| v.as_str()).map(String::from))
                })
                .unwrap_or_else(|| DEFAULT_VOICE.to_string());
            // Edge TTS voices follow the pattern "xx-XX-NameNeural" (≥2 hyphens).
            // Bare locales like "ko-KR" (1 hyphen) are not valid voice names.
            let voice = if raw_voice.matches('-').count() >= 2 {
                raw_voice
            } else {
                DEFAULT_VOICE.to_string()
            };

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

            let spoken = normalize_for_tts(text);

            let client = EdgeTtsClient::new()
                .map_err(|e| KittypawError::Skill(format!("TTS client error: {e}")))?;

            let result = client
                .synthesize(
                    &spoken,
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

/// Strip URLs, markdown syntax, and emojis so TTS reads clean natural language.
fn normalize_for_tts(text: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    static RE: OnceLock<Vec<(Regex, &str)>> = OnceLock::new();
    let rules = RE.get_or_init(|| {
        vec![
            // Markdown links [text](url) → text
            (Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap(), "$1"),
            // URLs
            (Regex::new(r"https?://\S+").unwrap(), ""),
            // Markdown headings
            (Regex::new(r"#{1,6}\s*").unwrap(), ""),
            // Bold/italic/strikethrough
            (Regex::new(r"[*_~`]{1,3}").unwrap(), ""),
            // Bullet markers at line start
            (Regex::new(r"(?m)^\s*[-*•]\s+").unwrap(), ""),
            // Emoji (Unicode Emoji range)
            (
                Regex::new(r"[\p{Emoji_Presentation}\p{Extended_Pictographic}]").unwrap(),
                "",
            ),
            // Collapse whitespace
            (Regex::new(r"[ \t]+").unwrap(), " "),
            // Collapse multiple newlines
            (Regex::new(r"\n{3,}").unwrap(), "\n\n"),
        ]
    });

    let mut result = text.to_string();
    for (re, replacement) in rules {
        result = re.replace_all(&result, *replacement).to_string();
    }
    result.trim().to_string()
}

fn uuid_short() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_strips_urls() {
        let input = "뉴스입니다 https://example.com/news 참고하세요";
        assert_eq!(normalize_for_tts(input), "뉴스입니다 참고하세요");
    }

    #[test]
    fn test_normalize_markdown_links() {
        let input = "[블룸버그](https://bloomberg.com) 기사";
        assert_eq!(normalize_for_tts(input), "블룸버그 기사");
    }

    #[test]
    fn test_normalize_strips_markdown_formatting() {
        let input = "## 제목\n**굵은 글씨**와 *기울임*";
        assert_eq!(normalize_for_tts(input), "제목\n굵은 글씨와 기울임");
    }

    #[test]
    fn test_normalize_strips_emojis() {
        let input = "🤖 AI 뉴스 요약 🔊";
        assert_eq!(normalize_for_tts(input), "AI 뉴스 요약");
    }

    #[test]
    fn test_normalize_collapses_whitespace() {
        let input = "hello   world\n\n\n\ntest";
        assert_eq!(normalize_for_tts(input), "hello world\n\ntest");
    }
}
