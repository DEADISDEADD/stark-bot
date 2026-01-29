//! Skills additional methods (unique to this module)
//! Core skills methods are in sqlite.rs

use rusqlite::Result as SqliteResult;

use crate::skills::DbSkill;
use super::super::Database;

impl Database {
    /// Get an enabled skill by name (more efficient than loading all skills)
    pub fn get_enabled_skill_by_name(&self, name: &str) -> SqliteResult<Option<DbSkill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, body, version, author, homepage, metadata, enabled, requires_tools, requires_binaries, arguments, tags, created_at, updated_at
             FROM skills WHERE name = ?1 AND enabled = 1 LIMIT 1"
        )?;

        let skill = stmt
            .query_row([name], |row| {
                let requires_tools_str: String = row.get(9)?;
                let requires_binaries_str: String = row.get(10)?;
                let arguments_str: String = row.get(11)?;
                let tags_str: String = row.get(12)?;

                Ok(DbSkill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    body: row.get(3)?,
                    version: row.get(4)?,
                    author: row.get::<_, Option<String>>(5)?,
                    homepage: row.get::<_, Option<String>>(6)?,
                    metadata: row.get::<_, Option<String>>(7)?,
                    enabled: row.get::<_, i32>(8)? != 0,
                    requires_tools: serde_json::from_str(&requires_tools_str).unwrap_or_default(),
                    requires_binaries: serde_json::from_str(&requires_binaries_str).unwrap_or_default(),
                    arguments: serde_json::from_str(&arguments_str).unwrap_or_default(),
                    tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                })
            })
            .ok();

        Ok(skill)
    }
}
