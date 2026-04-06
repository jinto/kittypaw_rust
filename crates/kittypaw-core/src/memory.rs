use crate::error::Result;

/// A hit from full-text search across execution history.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemorySearchHit {
    pub skill_name: String,
    pub result_summary: String,
    pub started_at: String,
    pub success: bool,
}

/// Abstraction over persistent memory storage.
///
/// Implemented by `Store` (SQLite-backed). The trait enables testing with
/// mocks and leaves room for alternative backends (Redis, Postgres, etc.).
pub trait MemoryProvider {
    /// Save a key-value pair to user context.
    fn memory_save(&self, key: &str, value: &str, source: &str) -> Result<()>;

    /// Recall entries matching a key prefix. Empty prefix returns all shared entries.
    fn memory_recall(&self, prefix: &str) -> Result<Vec<(String, String)>>;

    /// Full-text search across execution history.
    fn memory_search(&self, query: &str, limit: usize) -> Result<Vec<MemorySearchHit>>;

    /// Build context lines for LLM prompt injection.
    /// Returns user memories + recent failures as natural-language strings.
    fn memory_context_lines(&self) -> Result<Vec<String>>;
}
