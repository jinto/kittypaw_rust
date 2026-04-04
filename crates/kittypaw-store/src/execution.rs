use super::*;

impl Store {
    /// Record a skill execution
    #[allow(clippy::too_many_arguments)]
    pub fn record_execution(
        &self,
        skill_id: &str,
        skill_name: &str,
        started_at: &str,
        finished_at: &str,
        duration_ms: i64,
        result_summary: &str,
        success: bool,
        retry_count: i32,
        input_params: Option<&str>,
        usage_json: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO execution_history \
                 (skill_id, skill_name, started_at, finished_at, duration_ms, input_params, result_summary, success, retry_count, usage_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                skill_id,
                skill_name,
                started_at,
                finished_at,
                duration_ms,
                input_params,
                result_summary,
                success as i32,
                retry_count,
                usage_json,
            ],
        )?;
        Ok(())
    }

    /// Get recent executions (for dashboard activity log)
    pub fn recent_executions(&self, limit: usize) -> Result<Vec<ExecutionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, skill_id, skill_name, started_at, duration_ms, result_summary, success, retry_count, usage_json \
                 FROM execution_history ORDER BY started_at DESC LIMIT ?1",
        )?;

        let records = stmt
            .query_map(params![limit as i64], |row| {
                Ok(ExecutionRecord {
                    id: row.get(0)?,
                    skill_id: row.get(1)?,
                    skill_name: row.get(2)?,
                    started_at: row.get(3)?,
                    duration_ms: row.get(4)?,
                    result_summary: row.get(5)?,
                    success: row.get::<_, i32>(6)? != 0,
                    retry_count: row.get(7)?,
                    usage_json: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(records)
    }

    /// Get today's execution stats (for dashboard stat cards)
    pub fn today_stats(&self) -> Result<ExecutionStats> {
        let total_runs: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_history WHERE date(started_at) = date('now')",
            [],
            |row| row.get(0),
        )?;

        let successful: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_history WHERE date(started_at) = date('now') AND success = 1",
            [],
            |row| row.get(0),
        )?;

        let auto_retries: u32 = self.conn.query_row(
            "SELECT COALESCE(SUM(retry_count), 0) FROM execution_history WHERE date(started_at) = date('now')",
            [],
            |row| row.get(0),
        )?;

        let failed = total_runs.saturating_sub(successful);

        let total_tokens: u64 = {
            let mut stmt = self.conn.prepare(
                "SELECT usage_json FROM execution_history WHERE date(started_at) = date('now') AND usage_json IS NOT NULL",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.flatten()
                .map(|json_str| sum_usage_tokens(&json_str))
                .sum()
        };

        Ok(ExecutionStats {
            total_runs,
            successful,
            failed,
            auto_retries,
            total_tokens,
        })
    }

    /// Full-text search across execution history
    pub fn search_executions(&self, query: &str, limit: usize) -> Result<Vec<ExecutionRecord>> {
        // Sanitize: keep only alphanumeric, spaces, Korean (Hangul), and basic punctuation.
        // Strip FTS5 operators (AND, OR, NOT, NEAR, ^, *, :, (, ), -, +, etc.).
        // Cap at 200 chars. Reject empty input.
        let sanitized: String = query
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || *c == ' '
                    || *c == '.'
                    || *c == ','
                    || *c == '!'
                    || *c == '?'
                    || ('\u{AC00}'..='\u{D7A3}').contains(c)  // Hangul syllables
                    || ('\u{1100}'..='\u{11FF}').contains(c)  // Hangul jamo
                    || ('\u{3130}'..='\u{318F}').contains(c) // Hangul compatibility jamo
            })
            .take(200)
            .collect();
        let sanitized = sanitized.trim();
        if sanitized.is_empty() {
            return Ok(vec![]);
        }
        // Wrap as phrase to prevent any remaining operator misinterpretation.
        let safe_query = format!("\"{}\"", sanitized.replace('"', "\"\""));
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.skill_id, e.skill_name, e.started_at, e.duration_ms, e.result_summary, e.success, e.retry_count, e.usage_json \
             FROM execution_history e \
             JOIN execution_fts f ON e.id = f.rowid \
             WHERE execution_fts MATCH ?1 \
             ORDER BY e.started_at DESC LIMIT ?2",
        )?;
        let records = stmt
            .query_map(params![safe_query, limit as i64], |row| {
                Ok(ExecutionRecord {
                    id: row.get(0)?,
                    skill_id: row.get(1)?,
                    skill_name: row.get(2)?,
                    started_at: row.get(3)?,
                    duration_ms: row.get(4)?,
                    result_summary: row.get(5)?,
                    success: row.get::<_, i32>(6)? != 0,
                    retry_count: row.get(7)?,
                    usage_json: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(records)
    }

    /// Get execution count for a specific skill
    pub fn skill_execution_count(&self, skill_id: &str) -> Result<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM execution_history WHERE skill_id = ?1",
            params![skill_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Clean up old records (privacy: N-day retention)
    pub fn cleanup_old_executions(&self, days: u32) -> Result<u32> {
        let deleted = self.conn.execute(
            "DELETE FROM execution_history WHERE started_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )?;
        Ok(deleted as u32)
    }

    /// Delete conversation turns older than `max_age_days` days.
    /// `timestamp` is stored as Unix epoch seconds (string), so we cast to integer for comparison.
    pub fn cleanup_old_turns(&self, max_age_days: i64) -> Result<usize> {
        let cutoff = max_age_days * 86400;
        let deleted = self.conn.execute(
            "DELETE FROM conversations WHERE CAST(timestamp AS INTEGER) < (strftime('%s', 'now') - ?1)",
            params![cutoff],
        )?;
        Ok(deleted)
    }
}
