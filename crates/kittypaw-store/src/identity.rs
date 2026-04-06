use super::*;

/// A linked identity mapping a channel-specific user to a global user.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserIdentity {
    pub global_user_id: String,
    pub channel: String,
    pub channel_user_id: String,
    pub created_at: String,
}

impl Store {
    /// Link a channel user to a global user ID. Upserts — re-linking
    /// the same channel user to a different global user is allowed.
    pub fn link_identity(
        &self,
        global_user_id: &str,
        channel: &str,
        channel_user_id: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO user_identities (global_user_id, channel, channel_user_id) \
             VALUES (?1, ?2, ?3) \
             ON CONFLICT(channel, channel_user_id) \
             DO UPDATE SET global_user_id = excluded.global_user_id",
            params![global_user_id, channel, channel_user_id],
        )?;
        Ok(())
    }

    /// Resolve a channel user to their global user ID.
    /// Returns None if no identity link exists.
    pub fn resolve_user(&self, channel: &str, channel_user_id: &str) -> Result<Option<String>> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT global_user_id FROM user_identities \
             WHERE channel = ?1 AND channel_user_id = ?2",
            params![channel, channel_user_id],
            |row| row.get(0),
        );
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KittypawError::from(e)),
        }
    }

    /// List all identities linked to a global user.
    pub fn list_identities(&self, global_user_id: &str) -> Result<Vec<UserIdentity>> {
        let mut stmt = self.conn.prepare(
            "SELECT global_user_id, channel, channel_user_id, created_at \
             FROM user_identities WHERE global_user_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt
            .query_map(params![global_user_id], |row| {
                Ok(UserIdentity {
                    global_user_id: row.get(0)?,
                    channel: row.get(1)?,
                    channel_user_id: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Unlink a specific channel identity from a global user.
    /// If `channel_user_id` is Some, only that specific mapping is removed.
    /// If None, all mappings for that channel are removed.
    pub fn unlink_identity(
        &self,
        global_user_id: &str,
        channel: &str,
        channel_user_id: Option<&str>,
    ) -> Result<bool> {
        let deleted = if let Some(cuid) = channel_user_id {
            self.conn.execute(
                "DELETE FROM user_identities \
                 WHERE global_user_id = ?1 AND channel = ?2 AND channel_user_id = ?3",
                params![global_user_id, channel, cuid],
            )?
        } else {
            self.conn.execute(
                "DELETE FROM user_identities WHERE global_user_id = ?1 AND channel = ?2",
                params![global_user_id, channel],
            )?
        };
        Ok(deleted > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_test_store() -> Store {
        let path = {
            use std::sync::atomic::{AtomicU64, Ordering};
            static C: AtomicU64 = AtomicU64::new(0);
            std::env::temp_dir().join(format!(
                "kittypaw_identity_{}_{}.db",
                std::process::id(),
                C.fetch_add(1, Ordering::Relaxed)
            ))
        };
        Store::open(path.to_str().unwrap()).unwrap()
    }

    #[test]
    fn test_link_and_resolve() {
        let store = open_test_store();
        store.link_identity("u-abc", "telegram", "123").unwrap();

        let resolved = store.resolve_user("telegram", "123").unwrap();
        assert_eq!(resolved, Some("u-abc".to_string()));
    }

    #[test]
    fn test_resolve_unknown() {
        let store = open_test_store();
        let resolved = store.resolve_user("telegram", "999").unwrap();
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_link_upsert() {
        let store = open_test_store();
        store.link_identity("u-abc", "telegram", "123").unwrap();
        store.link_identity("u-xyz", "telegram", "123").unwrap();

        let resolved = store.resolve_user("telegram", "123").unwrap();
        assert_eq!(resolved, Some("u-xyz".to_string()));
    }

    #[test]
    fn test_list_identities() {
        let store = open_test_store();
        store.link_identity("u-abc", "telegram", "123").unwrap();
        store.link_identity("u-abc", "web", "ws-456").unwrap();
        store.link_identity("u-other", "telegram", "789").unwrap();

        let identities = store.list_identities("u-abc").unwrap();
        assert_eq!(identities.len(), 2);
        assert_eq!(identities[0].channel, "telegram");
        assert_eq!(identities[1].channel, "web");
    }

    #[test]
    fn test_unlink() {
        let store = open_test_store();
        store.link_identity("u-abc", "telegram", "123").unwrap();
        store.link_identity("u-abc", "web", "ws-456").unwrap();

        let deleted = store.unlink_identity("u-abc", "telegram", None).unwrap();
        assert!(deleted);

        let identities = store.list_identities("u-abc").unwrap();
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].channel, "web");
    }

    #[test]
    fn test_unlink_nonexistent() {
        let store = open_test_store();
        let deleted = store.unlink_identity("u-abc", "telegram", None).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_multiple_channels_same_global() {
        let store = open_test_store();
        store.link_identity("u-abc", "telegram", "123").unwrap();
        store.link_identity("u-abc", "desktop", "ws-1").unwrap();

        // Both resolve to the same global user
        assert_eq!(
            store.resolve_user("telegram", "123").unwrap(),
            Some("u-abc".to_string())
        );
        assert_eq!(
            store.resolve_user("desktop", "ws-1").unwrap(),
            Some("u-abc".to_string())
        );
    }
}
