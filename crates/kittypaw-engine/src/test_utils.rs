//! Shared test utilities for kittypaw-engine tests.
//!
//! Provides mock providers, store helpers, and event builders so individual
//! test modules don't each define the same helpers.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use kittypaw_core::config::Config;
use kittypaw_core::error::Result;
use kittypaw_core::types::{Event, EventType, LlmMessage};
use kittypaw_llm::provider::{LlmProvider, LlmResponse};
use kittypaw_store::Store;
use tokio::sync::Mutex;

// ── Temp DB path ──────────────────────────────────────────────────────────

/// Generates a unique temp file path for SQLite test databases.
/// Safe to call from concurrent tests.
pub fn temp_db_path() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut p = std::env::temp_dir();
    p.push(format!(
        "kittypaw_test_{}_{}.db",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    p
}

/// Opens a Store at the given path, panicking on failure (test helper).
pub fn open_store(path: &std::path::Path) -> Store {
    Store::open(path.to_str().unwrap()).unwrap()
}

/// Opens an in-memory Store (no cleanup needed).
pub fn make_in_memory_store() -> Arc<Mutex<Store>> {
    Arc::new(Mutex::new(
        Store::open(":memory:").expect("in-memory store"),
    ))
}

// ── Mock LLM Provider ─────────────────────────────────────────────────────

/// A mock LLM provider that returns a fixed JS code string.
/// Use `MockJsProvider::new("return 42;")` in tests.
pub struct MockJsProvider {
    js_code: String,
}

impl MockJsProvider {
    pub fn new(js: &str) -> Self {
        Self {
            js_code: js.to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for MockJsProvider {
    async fn generate(&self, _messages: &[LlmMessage]) -> Result<LlmResponse> {
        Ok(LlmResponse::text_only(self.js_code.clone()))
    }
}

/// A mock LLM provider that panics if called — proves a code path avoids LLM.
pub struct PanicProvider;

#[async_trait]
impl LlmProvider for PanicProvider {
    async fn generate(&self, _messages: &[LlmMessage]) -> Result<LlmResponse> {
        panic!("LLM should not be called in this test");
    }
}

/// A mock LLM provider that returns different responses in sequence.
/// On exhaustion, returns the last response repeatedly.
pub struct SequentialMockProvider {
    responses: std::sync::Mutex<std::collections::VecDeque<String>>,
}

impl SequentialMockProvider {
    pub fn new(responses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses.into_iter().map(Into::into).collect()),
        }
    }
}

#[async_trait]
impl LlmProvider for SequentialMockProvider {
    async fn generate(&self, _messages: &[LlmMessage]) -> Result<LlmResponse> {
        let mut q = self.responses.lock().unwrap();
        let code = if q.len() > 1 {
            q.pop_front().unwrap()
        } else {
            q.front().cloned().unwrap_or_default()
        };
        Ok(LlmResponse::text_only(code))
    }
}

// ── Config helpers ────────────────────────────────────────────────────────

/// Test config with freeform_fallback enabled so natural language reaches agent_loop.
pub fn test_config() -> Config {
    let mut config = Config::default();
    config.freeform_fallback = true;
    config
}

// ── Event helpers ─────────────────────────────────────────────────────────

pub fn desktop_event(text: &str) -> Event {
    Event {
        event_type: EventType::Desktop,
        payload: serde_json::json!({
            "text": text,
            "workspace_id": "test",
        }),
    }
}

pub fn telegram_event(text: &str, chat_id: &str) -> Event {
    Event {
        event_type: EventType::Telegram,
        payload: serde_json::json!({
            "text": text,
            "chat_id": chat_id,
            "from_name": "test_user",
        }),
    }
}
