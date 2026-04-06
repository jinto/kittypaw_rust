use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditRecord {
    pub id: i64,
    pub event_type: String,
    pub detail: String,
    pub severity: String,
    pub created_at: String,
}

impl Store {
    /// Record a security audit event.
    pub fn record_audit(&self, event_type: &str, detail: &str, severity: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO audit_log (event_type, detail, severity) VALUES (?1, ?2, ?3)",
            params![event_type, detail, severity],
        )?;
        Ok(())
    }

    /// Query recent audit events, newest first.
    pub fn recent_audit_events(&self, limit: usize) -> Result<Vec<AuditRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_type, detail, severity, created_at \
             FROM audit_log ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(AuditRecord {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    detail: row.get(2)?,
                    severity: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Count audit events by severity (for dashboard).
    pub fn audit_summary(&self) -> Result<HashMap<String, u32>> {
        let mut stmt = self
            .conn
            .prepare("SELECT severity, COUNT(*) FROM audit_log GROUP BY severity")?;
        let map: HashMap<String, u32> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(map)
    }

    /// Delete audit events older than N days.
    pub fn cleanup_old_audit(&self, days: u32) -> Result<u32> {
        let deleted = self.conn.execute(
            "DELETE FROM audit_log WHERE created_at < datetime('now', ?1)",
            params![format!("-{days} days")],
        )?;
        Ok(deleted as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (Store, std::path::PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "kp_audit_test_{}_{}.db",
            std::process::id(),
            CTR.fetch_add(1, Ordering::Relaxed)
        ));
        let store = Store::open(p.to_str().unwrap()).unwrap();
        (store, p)
    }

    #[test]
    fn test_audit_roundtrip() {
        let (store, p) = temp_store();
        store
            .record_audit("ssrf_blocked", "blocked 192.168.1.1", "warn")
            .unwrap();
        store
            .record_audit("capability_denied", "Shell.exec denied for skill-x", "info")
            .unwrap();

        let events = store.recent_audit_events(10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "capability_denied"); // newest first

        let summary = store.audit_summary().unwrap();
        assert_eq!(summary.get("warn"), Some(&1));
        assert_eq!(summary.get("info"), Some(&1));

        let _ = std::fs::remove_file(&p);
    }
}
