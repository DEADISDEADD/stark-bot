//! Skill embedding and association logic
//! Provides semantic skill discovery via vector embeddings

use crate::db::Database;
use crate::memory::EmbeddingGenerator;
use crate::memory::vector_search;
use crate::skills::types::DbSkill;
use std::sync::Arc;

/// Build embedding text for a skill (concise representation for vector search)
pub fn build_skill_embedding_text(skill: &DbSkill) -> String {
    let tags = if skill.tags.is_empty() {
        String::new()
    } else {
        format!(". Tags: {}", skill.tags.join(", "))
    };
    format!("{}: {}{}", skill.name, skill.description, tags)
}

/// Backfill embeddings for all enabled skills that don't have one yet.
/// Returns the number of embeddings generated.
pub async fn backfill_skill_embeddings(
    db: &Arc<Database>,
    embedding_gen: &Arc<dyn EmbeddingGenerator + Send + Sync>,
) -> Result<usize, String> {
    let missing_ids = db.list_skills_without_embeddings(100)
        .map_err(|e| format!("Failed to list skills without embeddings: {}", e))?;

    if missing_ids.is_empty() {
        return Ok(0);
    }

    let mut count = 0;
    for skill_id in &missing_ids {
        let skill = match db.get_skill_by_id(*skill_id) {
            Ok(Some(s)) => s,
            _ => continue,
        };

        let text = build_skill_embedding_text(&skill);
        match embedding_gen.generate(&text).await {
            Ok(embedding) => {
                let dims = embedding.len() as i32;
                if let Err(e) = db.upsert_skill_embedding(*skill_id, &embedding, "remote", dims) {
                    log::warn!("[SKILL-EMB] Failed to store embedding for skill {}: {}", skill.name, e);
                } else {
                    count += 1;
                    log::debug!("[SKILL-EMB] Generated embedding for skill '{}'", skill.name);
                }
            }
            Err(e) => {
                log::warn!("[SKILL-EMB] Failed to generate embedding for skill '{}': {}", skill.name, e);
            }
        }
    }

    log::info!("[SKILL-EMB] Backfilled {} skill embeddings", count);
    Ok(count)
}

/// Search skills by semantic similarity to a query string.
/// Returns matching skills with their similarity scores.
pub async fn search_skills(
    db: &Arc<Database>,
    embedding_gen: &Arc<dyn EmbeddingGenerator + Send + Sync>,
    query: &str,
    limit: usize,
    threshold: f32,
) -> Result<Vec<(DbSkill, f32)>, String> {
    // Generate query embedding
    let query_embedding = embedding_gen.generate(query).await?;

    // Load all skill embeddings
    let candidates = db.list_skill_embeddings()
        .map_err(|e| format!("Failed to list skill embeddings: {}", e))?;

    if candidates.is_empty() {
        return Ok(vec![]);
    }

    // Find similar using vector search
    let results = vector_search::find_similar(&query_embedding, &candidates, limit, threshold);

    // Map result IDs back to DbSkill objects
    let mut skills_with_scores = Vec::new();
    for result in results {
        // result.memory_id is actually skill_id here (same field name from VectorSearchResult)
        if let Ok(Some(skill)) = db.get_skill_by_id(result.memory_id) {
            if skill.enabled {
                skills_with_scores.push((skill, result.similarity));
            }
        }
    }

    Ok(skills_with_scores)
}

/// Rebuild all skill associations from embeddings.
/// Deletes existing associations, backfills missing embeddings,
/// then creates associations for skill pairs above the similarity threshold.
pub async fn rebuild_skill_associations(
    db: &Arc<Database>,
    embedding_gen: &Arc<dyn EmbeddingGenerator + Send + Sync>,
    threshold: f32,
) -> Result<usize, String> {
    // Delete all existing associations
    db.delete_all_skill_associations()
        .map_err(|e| format!("Failed to delete existing associations: {}", e))?;

    // Backfill any missing embeddings
    backfill_skill_embeddings(db, embedding_gen).await?;

    // Load all skill embeddings
    let all_embeddings = db.list_skill_embeddings()
        .map_err(|e| format!("Failed to list skill embeddings: {}", e))?;

    if all_embeddings.len() < 2 {
        return Ok(0);
    }

    // Load all skills for tag comparison
    let all_skills: std::collections::HashMap<i64, DbSkill> = db.list_enabled_skills()
        .map_err(|e| format!("Failed to list skills: {}", e))?
        .into_iter()
        .filter_map(|s| s.id.map(|id| (id, s)))
        .collect();

    let mut created = 0;

    // Compare all pairs
    for i in 0..all_embeddings.len() {
        for j in (i + 1)..all_embeddings.len() {
            let (id_a, ref emb_a) = all_embeddings[i];
            let (id_b, ref emb_b) = all_embeddings[j];

            let similarity = vector_search::cosine_similarity(emb_a, emb_b);
            if similarity < threshold {
                continue;
            }

            // Classify association type based on shared tags
            let assoc_type = if let (Some(skill_a), Some(skill_b)) = (all_skills.get(&id_a), all_skills.get(&id_b)) {
                let shared_tags = skill_a.tags.iter().any(|t| skill_b.tags.contains(t));
                if shared_tags { "complement" } else { "related" }
            } else {
                "related"
            };

            if let Err(e) = db.create_skill_association(id_a, id_b, assoc_type, similarity as f64, None) {
                log::warn!("[SKILL-ASSOC] Failed to create association ({} -> {}): {}", id_a, id_b, e);
            } else {
                created += 1;
            }
        }
    }

    log::info!("[SKILL-ASSOC] Rebuilt {} skill associations", created);
    Ok(created)
}
