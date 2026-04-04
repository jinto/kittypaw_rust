use super::*;

impl Store {
    pub fn storage_get(&self, namespace: &str, key: &str) -> Result<Option<String>> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT value FROM skill_storage WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KittypawError::from(e)),
        }
    }

    pub fn storage_set(&self, namespace: &str, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO skill_storage (namespace, key, value) VALUES (?1, ?2, ?3)",
            params![namespace, key, value],
        )?;
        Ok(())
    }

    pub fn storage_delete(&self, namespace: &str, key: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM skill_storage WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
        )?;
        Ok(())
    }

    pub fn storage_list(&self, namespace: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key FROM skill_storage WHERE namespace = ?1")?;
        let keys: Vec<String> = stmt
            .query_map(params![namespace], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(keys)
    }
}
