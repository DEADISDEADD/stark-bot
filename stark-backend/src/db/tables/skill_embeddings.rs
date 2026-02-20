//! Database operations for skill_embeddings table
//! Stores vector embeddings for skills (semantic skill discovery)

use crate::db::Database;
use super::memory_embeddings::{embedding_to_blob, blob_to_embedding};

impl Database {
    /// Upsert an embedding for a skill
    pub fn upsert_skill_embedding(
        &self,
        skill_id: i64,
        embedding: &[f32],
        model: &str,
        dimensions: i32,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        let blob = embedding_to_blob(embedding);
        conn.execute(
            "INSERT INTO skill_embeddings (skill_id, embedding, model, dimensions, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(skill_id) DO UPDATE SET
                embedding = excluded.embedding,
                model = excluded.model,
                dimensions = excluded.dimensions,
                updated_at = datetime('now')",
            rusqlite::params![skill_id, blob, model, dimensions],
        )?;
        Ok(())
    }

    /// Get embedding for a specific skill
    pub fn get_skill_embedding(&self, skill_id: i64) -> Result<Option<(Vec<f32>, String, i32)>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT embedding, model, dimensions FROM skill_embeddings WHERE skill_id = ?1"
        )?;
        let result = stmt.query_row(rusqlite::params![skill_id], |row| {
            let blob: Vec<u8> = row.get(0)?;
            let model: String = row.get(1)?;
            let dimensions: i32 = row.get(2)?;
            Ok((blob_to_embedding(&blob), model, dimensions))
        });
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get all skill embeddings (for brute-force vector search)
    pub fn list_skill_embeddings(&self) -> Result<Vec<(i64, Vec<f32>)>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT skill_id, embedding FROM skill_embeddings"
        )?;
        let rows = stmt.query_map([], |row| {
            let skill_id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((skill_id, blob_to_embedding(&blob)))
        })?;
        rows.collect()
    }

    /// List skill IDs that have no embedding yet
    pub fn list_skills_without_embeddings(&self, limit: i32) -> Result<Vec<i64>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT s.id FROM skills s
             LEFT JOIN skill_embeddings se ON s.id = se.skill_id
             WHERE se.skill_id IS NULL AND s.enabled = 1
             ORDER BY s.id
             LIMIT ?1"
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| row.get(0))?;
        rows.collect()
    }

    /// Count total skill embeddings
    pub fn count_skill_embeddings(&self) -> Result<i64, rusqlite::Error> {
        let conn = self.conn();
        conn.query_row(
            "SELECT COUNT(*) FROM skill_embeddings",
            [],
            |row| row.get(0),
        )
    }
}
