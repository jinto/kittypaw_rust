use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Checkpoint {
    pub id: i64,
    pub agent_id: String,
    pub label: String,
    pub conv_row_id: i64,
    pub created_at: String,
}

impl Store {
    /// Snapshot the current conversation position.
    /// Returns the checkpoint ID.
    pub fn create_checkpoint(&self, agent_id: &str, label: &str) -> Result<i64> {
        // Get the current max conversation row ID for this agent (0 if empty)
        let max_id: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) FROM conversations WHERE agent_id = ?1",
                params![agent_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO agent_checkpoints (agent_id, label, conv_row_id) VALUES (?1, ?2, ?3)",
            params![agent_id, label, max_id],
        )?;

        let checkpoint_id = self.conn.last_insert_rowid();
        Ok(checkpoint_id)
    }

    /// Roll back: delete all conversation turns added after the checkpoint.
    /// Returns the number of turns deleted.
    pub fn rollback_to_checkpoint(&self, checkpoint_id: i64) -> Result<usize> {
        let (agent_id, conv_row_id): (String, i64) = self
            .conn
            .query_row(
                "SELECT agent_id, conv_row_id FROM agent_checkpoints WHERE id = ?1",
                params![checkpoint_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| KittypawError::Store(format!("checkpoint {checkpoint_id} not found")))?;

        let deleted = self.conn.execute(
            "DELETE FROM conversations WHERE agent_id = ?1 AND id > ?2",
            params![agent_id, conv_row_id],
        )?;

        // Remove checkpoints created after this one (they reference deleted state)
        self.conn.execute(
            "DELETE FROM agent_checkpoints WHERE agent_id = ?1 AND id > ?2",
            params![agent_id, checkpoint_id],
        )?;

        Ok(deleted)
    }

    /// List checkpoints for an agent, newest first.
    pub fn list_checkpoints(&self, agent_id: &str) -> Result<Vec<Checkpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, label, conv_row_id, created_at \
             FROM agent_checkpoints WHERE agent_id = ?1 \
             ORDER BY id DESC",
        )?;
        let rows = stmt
            .query_map(params![agent_id], |row| {
                Ok(Checkpoint {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    label: row.get(2)?,
                    conv_row_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Delete a single checkpoint (without rolling back).
    pub fn delete_checkpoint(&self, checkpoint_id: i64) -> Result<bool> {
        let deleted = self.conn.execute(
            "DELETE FROM agent_checkpoints WHERE id = ?1",
            params![checkpoint_id],
        )?;
        Ok(deleted > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kittypaw_core::types::{ConversationTurn, Role};

    fn temp_store() -> (Store, std::path::PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "kp_ckpt_test_{}_{}.db",
            std::process::id(),
            CTR.fetch_add(1, Ordering::Relaxed)
        ));
        let store = Store::open(p.to_str().unwrap()).unwrap();
        (store, p)
    }

    fn add_msg(store: &Store, agent: &str, text: &str) {
        let turn = ConversationTurn {
            role: Role::User,
            content: text.to_string(),
            code: None,
            result: None,
            timestamp: "1000".to_string(),
        };
        store.add_turn(agent, &turn).unwrap();
    }

    #[test]
    fn test_checkpoint_and_rollback() {
        let (store, p) = temp_store();
        let agent = "agent-ckpt";
        let state = kittypaw_core::types::AgentState::new(agent, "sys");
        store.save_state(&state).unwrap();

        add_msg(&store, agent, "msg-1");
        add_msg(&store, agent, "msg-2");

        let cp = store.create_checkpoint(agent, "before-experiment").unwrap();
        assert!(cp > 0);

        // Add more turns after checkpoint
        add_msg(&store, agent, "msg-3");
        add_msg(&store, agent, "msg-4");

        let all = store.recent_turns_all(agent).unwrap();
        assert_eq!(all.len(), 4);

        // Rollback
        let deleted = store.rollback_to_checkpoint(cp).unwrap();
        assert_eq!(deleted, 2);

        let after = store.recent_turns_all(agent).unwrap();
        assert_eq!(after.len(), 2);
        assert_eq!(after[0].content, "msg-1");
        assert_eq!(after[1].content, "msg-2");

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_list_and_delete_checkpoints() {
        let (store, p) = temp_store();
        let agent = "agent-list";
        let state = kittypaw_core::types::AgentState::new(agent, "sys");
        store.save_state(&state).unwrap();

        add_msg(&store, agent, "msg-1");
        let cp1 = store.create_checkpoint(agent, "cp-1").unwrap();

        add_msg(&store, agent, "msg-2");
        let cp2 = store.create_checkpoint(agent, "cp-2").unwrap();

        let list = store.list_checkpoints(agent).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, cp2); // newest first
        assert_eq!(list[1].id, cp1);

        store.delete_checkpoint(cp1).unwrap();
        let list = store.list_checkpoints(agent).unwrap();
        assert_eq!(list.len(), 1);

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_rollback_cascades_later_checkpoints() {
        let (store, p) = temp_store();
        let agent = "agent-cascade";
        let state = kittypaw_core::types::AgentState::new(agent, "sys");
        store.save_state(&state).unwrap();

        add_msg(&store, agent, "msg-1");
        let cp1 = store.create_checkpoint(agent, "cp-1").unwrap();

        add_msg(&store, agent, "msg-2");
        let _cp2 = store.create_checkpoint(agent, "cp-2").unwrap();

        // Rolling back to cp1 should also remove cp2
        store.rollback_to_checkpoint(cp1).unwrap();
        let list = store.list_checkpoints(agent).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, cp1);

        let _ = std::fs::remove_file(&p);
    }
}
