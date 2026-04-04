use super::*;

impl Store {
    pub fn save_workspace(&self, id: &str, name: &str, root_path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO workspaces (id, name, root_path, last_opened_at) \
                 VALUES (?1, ?2, ?3, datetime('now'))",
            params![id, name, root_path],
        )?;
        Ok(())
    }

    // ── Permission CRUD ────────────────────────────────────────────────────

    pub fn save_file_rule(&self, rule: &FilePermissionRule) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO permission_file_rules \
                 (id, workspace_id, path_pattern, is_exception, can_read, can_write, can_delete) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rule.id,
                rule.workspace_id,
                rule.path_pattern,
                rule.is_exception as i32,
                rule.can_read as i32,
                rule.can_write as i32,
                rule.can_delete as i32,
            ],
        )?;
        Ok(())
    }

    pub fn list_file_rules(&self, workspace_id: &str) -> Result<Vec<FilePermissionRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, path_pattern, is_exception, can_read, can_write, can_delete \
                 FROM permission_file_rules WHERE workspace_id = ?1",
        )?;

        let rules = stmt
            .query_map(params![workspace_id], |row| {
                Ok(FilePermissionRule {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    path_pattern: row.get(2)?,
                    is_exception: row.get::<_, i32>(3)? != 0,
                    can_read: row.get::<_, i32>(4)? != 0,
                    can_write: row.get::<_, i32>(5)? != 0,
                    can_delete: row.get::<_, i32>(6)? != 0,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rules)
    }

    pub fn delete_file_rule(&self, rule_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM permission_file_rules WHERE id = ?1",
            params![rule_id],
        )?;
        Ok(())
    }

    pub fn save_network_rule(&self, rule: &NetworkPermissionRule) -> Result<()> {
        let methods_json =
            serde_json::to_string(&rule.allowed_methods).map_err(KittypawError::Json)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO permission_network_rules \
                 (id, workspace_id, domain_pattern, allowed_methods) \
                 VALUES (?1, ?2, ?3, ?4)",
            params![
                rule.id,
                rule.workspace_id,
                rule.domain_pattern,
                methods_json
            ],
        )?;
        Ok(())
    }

    pub fn list_network_rules(&self, workspace_id: &str) -> Result<Vec<NetworkPermissionRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, domain_pattern, allowed_methods \
                 FROM permission_network_rules WHERE workspace_id = ?1",
        )?;

        let rules = stmt
            .query_map(params![workspace_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|(id, workspace_id, domain_pattern, methods_json)| {
                let allowed_methods: Vec<HttpMethod> =
                    serde_json::from_str(&methods_json).unwrap_or_default();
                NetworkPermissionRule {
                    id,
                    workspace_id,
                    domain_pattern,
                    allowed_methods,
                }
            })
            .collect();

        Ok(rules)
    }

    pub fn delete_network_rule(&self, rule_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM permission_network_rules WHERE id = ?1",
            params![rule_id],
        )?;
        Ok(())
    }

    pub fn save_global_path(&self, global_path: &GlobalPath) -> Result<()> {
        let access_type_str = match global_path.access_type {
            AccessType::Read => "read",
            AccessType::Write => "write",
        };
        self.conn.execute(
            "INSERT OR REPLACE INTO permission_global_paths (id, path, access_type) \
                 VALUES (?1, ?2, ?3)",
            params![global_path.id, global_path.path, access_type_str],
        )?;
        Ok(())
    }

    pub fn list_global_paths(&self) -> Result<Vec<GlobalPath>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, access_type FROM permission_global_paths")?;

        let paths = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|(id, path, access_type_str)| {
                let access_type = if access_type_str == "write" {
                    AccessType::Write
                } else {
                    AccessType::Read
                };
                GlobalPath {
                    id,
                    path,
                    access_type,
                }
            })
            .collect();

        Ok(paths)
    }

    pub fn delete_global_path(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM permission_global_paths WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn load_permission_profile(&self, workspace_id: &str) -> Result<PermissionProfile> {
        let file_rules = self.list_file_rules(workspace_id)?;
        let network_rules = self.list_network_rules(workspace_id)?;
        let global_paths = self.list_global_paths()?;
        Ok(PermissionProfile {
            workspace_id: workspace_id.to_string(),
            file_rules,
            network_rules,
            global_paths,
        })
    }
}
