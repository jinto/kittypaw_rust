use kittypaw_core::error::{KittypawError, LlmErrorKind};

/// Classify a reqwest transport error into the appropriate LLM error kind.
/// Connection and timeout failures map to `Network`; everything else to `Other`.
pub fn classify_reqwest_error(e: &reqwest::Error) -> KittypawError {
    let kind = if e.is_connect() || e.is_timeout() {
        LlmErrorKind::Network
    } else {
        LlmErrorKind::Other
    };
    KittypawError::Llm {
        kind,
        message: format!("HTTP error: {e}"),
    }
}

/// Result of checking an HTTP response status for retryable conditions.
pub enum StatusAction {
    /// Response is ready for the caller to consume (success or non-retryable error).
    Success(reqwest::Response),
    /// Caller should sleep for this duration, then retry.
    Retry(std::time::Duration),
    /// Retries exhausted for a retryable status code.
    Fatal(KittypawError),
}

/// Classify an HTTP response status into a retry action.
/// Handles 429 (rate limit) and 5xx (server error) with exponential backoff.
/// Increments `retries` on each retryable status and returns `Fatal` when exhausted.
pub fn handle_response_status(
    response: reqwest::Response,
    retries: &mut u32,
    max_retries: u32,
) -> StatusAction {
    let status = response.status();

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        *retries += 1;
        if *retries > max_retries {
            return StatusAction::Fatal(KittypawError::Llm {
                kind: LlmErrorKind::RateLimit,
                message: "Rate limited after max retries".into(),
            });
        }
        let delay = std::time::Duration::from_millis(1000 * 2u64.pow(*retries));
        tracing::warn!("Rate limited, retrying in {:?}", delay);
        return StatusAction::Retry(delay);
    }

    if status.is_server_error() {
        *retries += 1;
        if *retries > max_retries {
            return StatusAction::Fatal(KittypawError::Llm {
                kind: LlmErrorKind::Other,
                message: format!("Server error {status} after max retries"),
            });
        }
        let delay = std::time::Duration::from_millis(1000 * 2u64.pow(*retries));
        tracing::warn!("Server error {status}, retrying in {:?}", delay);
        return StatusAction::Retry(delay);
    }

    // Non-retryable: either success or a client error the caller must classify
    StatusAction::Success(response)
}

/// Strip markdown code fences from LLM-generated code.
/// Handles ```javascript, ```js, and bare ``` fences.
pub fn strip_code_fences(code: &str) -> String {
    let trimmed = code.trim();

    // Check for opening code fence
    if let Some(rest) = trimmed.strip_prefix("```") {
        // Skip the language identifier line
        let after_lang = if let Some(pos) = rest.find('\n') {
            &rest[pos + 1..]
        } else {
            return String::new();
        };

        // Remove closing fence
        if let Some(content) = after_lang.strip_suffix("```") {
            return content.trim().to_string();
        }
        // No closing fence — return as-is
        return after_lang.trim().to_string();
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_js_fence() {
        let input = "```javascript\nconsole.log('hello');\n```";
        assert_eq!(strip_code_fences(input), "console.log('hello');");
    }

    #[test]
    fn test_strip_bare_fence() {
        let input = "```\nconst x = 1;\n```";
        assert_eq!(strip_code_fences(input), "const x = 1;");
    }

    #[test]
    fn test_no_fence() {
        let input = "console.log('hello');";
        assert_eq!(strip_code_fences(input), "console.log('hello');");
    }

    #[test]
    fn test_strip_with_whitespace() {
        let input = "  ```js\n  const x = 1;\n  ```  ";
        assert_eq!(strip_code_fences(input), "const x = 1;");
    }
}
