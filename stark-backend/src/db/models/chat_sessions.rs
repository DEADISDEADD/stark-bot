//! Chat session context management methods (compaction)
//! Core chat session methods are in sqlite.rs

use chrono::Utc;
use rusqlite::Result as SqliteResult;

use crate::models::{MessageRole, SessionMessage};
use super::super::Database;

impl Database {
    // ============================================
    // Context Management methods (unique to this module)
    // ============================================

    /// Update the context token count for a session
    pub fn update_session_context_tokens(&self, session_id: i64, context_tokens: i32) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chat_sessions SET context_tokens = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![context_tokens, &now, session_id],
        )?;
        Ok(())
    }

    /// Set the compaction ID for a session (after compaction occurs)
    pub fn set_session_compaction(&self, session_id: i64, compaction_id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chat_sessions SET compaction_id = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![compaction_id, &now, session_id],
        )?;
        Ok(())
    }

    /// Get oldest messages for compaction (excludes most recent messages)
    pub fn get_messages_for_compaction(&self, session_id: i64, keep_recent: i32) -> SqliteResult<Vec<SessionMessage>> {
        let conn = self.conn.lock().unwrap();

        // Get total count first
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM session_messages WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )?;

        let to_compact = (total as i32).saturating_sub(keep_recent);
        if to_compact <= 0 {
            return Ok(vec![]);
        }

        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, user_id, user_name, platform_message_id, tokens_used, created_at
             FROM session_messages WHERE session_id = ?1 ORDER BY created_at ASC LIMIT ?2",
        )?;

        let messages = stmt
            .query_map(rusqlite::params![session_id, to_compact], |row| {
                let created_at_str: String = row.get(8)?;
                let role_str: String = row.get(2)?;

                Ok(SessionMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: MessageRole::from_str(&role_str).unwrap_or(MessageRole::User),
                    content: row.get(3)?,
                    user_id: row.get(4)?,
                    user_name: row.get(5)?,
                    platform_message_id: row.get(6)?,
                    tokens_used: row.get(7)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    }

    /// Delete old messages after compaction (keeps the most recent messages)
    pub fn delete_compacted_messages(&self, session_id: i64, keep_recent: i32) -> SqliteResult<i32> {
        let conn = self.conn.lock().unwrap();

        // Get IDs of messages to delete (all except the most recent)
        let deleted = conn.execute(
            "DELETE FROM session_messages WHERE session_id = ?1 AND id NOT IN (
                SELECT id FROM session_messages WHERE session_id = ?1 ORDER BY created_at DESC LIMIT ?2
            )",
            rusqlite::params![session_id, keep_recent],
        )?;

        Ok(deleted as i32)
    }

    /// Get the compaction summary for a session (if any)
    pub fn get_session_compaction_summary(&self, session_id: i64) -> SqliteResult<Option<String>> {
        let conn = self.conn.lock().unwrap();

        // First get the compaction_id from the session
        let compaction_id: Option<i64> = conn.query_row(
            "SELECT compaction_id FROM chat_sessions WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        ).ok().flatten();

        let Some(compaction_id) = compaction_id else {
            return Ok(None);
        };

        // Get the compaction memory content
        let content: Option<String> = conn.query_row(
            "SELECT content FROM memories WHERE id = ?1",
            [compaction_id],
            |row| row.get(0),
        ).ok();

        Ok(content)
    }
}
