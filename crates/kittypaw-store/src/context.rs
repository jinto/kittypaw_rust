use super::*;

impl Store {
    /// Get a user context value by key
    pub fn get_user_context(&self, key: &str) -> Result<Option<String>> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT value FROM user_context WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KittypawError::from(e)),
        }
    }

    /// List all user context entries whose key starts with the given prefix.
    /// Returns Vec<(key, value)>.
    pub fn list_user_context_prefix(&self, prefix: &str) -> Result<Vec<(String, String)>> {
        let like_pattern = format!("{}%", prefix);
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM user_context WHERE key LIKE ?1")?;
        let rows: Vec<(String, String)> = stmt
            .query_map(params![like_pattern], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// List user context entries that are shareable across skills.
    /// Excludes internal keys (default:*, suggest_*, schedule_*, onboarding*).
    pub fn list_shared_context(&self) -> Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value FROM user_context \
             WHERE key NOT LIKE 'default:%' \
               AND key NOT LIKE 'suggest_%' \
               AND key NOT LIKE 'schedule_%' \
               AND key NOT LIKE 'onboarding%' \
               AND key NOT LIKE 'failure_hint:%'",
        )?;
        let map: HashMap<String, String> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(map)
    }

    /// Set a user context value
    pub fn set_user_context(&self, key: &str, value: &str, source: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_context (key, value, source, updated_at) \
                 VALUES (?1, ?2, ?3, datetime('now'))",
            params![key, value, source],
        )?;
        Ok(())
    }

    /// Find config keys where the same value was used 3+ times for a skill.
    /// Returns Vec<(key, value)> pairs that should become defaults.
    pub fn detect_param_patterns(&self, skill_id: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT input_params FROM execution_history \
                 WHERE skill_id = ?1 AND input_params IS NOT NULL \
                 ORDER BY started_at DESC LIMIT 50",
        )?;

        let params_rows: Vec<String> = stmt
            .query_map(params![skill_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        // Count occurrences of each (key, value) pair
        let mut counts: std::collections::HashMap<(String, String), u32> =
            std::collections::HashMap::new();

        for params_json in &params_rows {
            if let Ok(serde_json::Value::Object(map)) =
                serde_json::from_str::<serde_json::Value>(params_json)
            {
                for (k, v) in &map {
                    if let Some(val_str) = v.as_str() {
                        *counts.entry((k.clone(), val_str.to_string())).or_insert(0) += 1;
                    }
                }
            }
        }

        // Return keys where a single value appears >= 3 times,
        // excluding any key that looks like a secret
        let mut patterns: Vec<(String, String)> = counts
            .into_iter()
            .filter(|((k, _), count)| {
                *count >= 3
                    && !k.contains("token")
                    && !k.contains("secret")
                    && !k.contains("api_key")
            })
            .map(|((k, v), _)| (k, v))
            .collect();
        patterns.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(patterns)
    }

    /// Detect if a skill is being run manually at consistent times.
    /// Returns Some(suggested_cron) if a pattern is found, None otherwise.
    pub fn detect_time_pattern(&self, skill_id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT started_at FROM execution_history \
                 WHERE skill_id = ?1 \
                 ORDER BY started_at DESC LIMIT 7",
        )?;

        let times: Vec<String> = stmt
            .query_map(params![skill_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if times.len() < 3 {
            return Ok(None);
        }

        // Extract the hour from each started_at timestamp
        let hours: Vec<u32> = times
            .iter()
            .filter_map(|s| {
                // Handles "YYYY-MM-DDTHH:MM:SS" and "YYYY-MM-DD HH:MM:SS"
                let time_part = if let Some(pos) = s.find('T') {
                    &s[pos + 1..]
                } else if let Some(pos) = s.find(' ') {
                    &s[pos + 1..]
                } else {
                    return None;
                };
                time_part[..2].parse::<u32>().ok()
            })
            .collect();

        if hours.len() < 3 {
            return Ok(None);
        }

        // Find the most common hour (allowing +/-1 tolerance)
        let mut hour_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for &h in &hours {
            *hour_counts.entry(h).or_insert(0) += 1;
        }

        // Check if any hour bucket (with +/-1 window) has 3+ hits
        for &base_hour in hour_counts.keys() {
            let window_count: u32 = hours
                .iter()
                .filter(|&&h| {
                    let diff = h.abs_diff(base_hour);
                    diff.min(24 - diff) <= 1
                })
                .count() as u32;
            if window_count >= 3 {
                // Suggest a daily cron at this hour
                return Ok(Some(format!("0 0 {} * * *", base_hour)));
            }
        }

        Ok(None)
    }
}
