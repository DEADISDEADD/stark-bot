//! Shared FTS5 query normalization utilities.
//!
//! Provides a single `normalize_fts_query()` function used by:
//! - System prompt memory pre-injection (fallback path)
//! - Hybrid search engine (`fts_search`)
//! - User-facing FTS search (`search_memories_fts_user`)

/// Normalize a raw query string into an FTS5 MATCH expression with stemming.
///
/// Steps:
/// 1. Split on whitespace, trim punctuation, lowercase
/// 2. Filter stop words (English) and short tokens (< 2 chars)
/// 3. Take first 8 significant words
/// 4. Apply `simple_stem()` to each word
/// 5. Produce prefix-wildcard OR query: `stem1* OR original1* OR stem2*`
///
/// Returns an empty string if no significant tokens remain.
pub fn normalize_fts_query(query: &str) -> String {
    let stop_words = stop_words::get(stop_words::LANGUAGE::English);

    query
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| w.len() >= 2 && !stop_words.contains(&w.as_str()))
        .take(8)
        .map(|w| {
            let stemmed = simple_stem(&w);
            if stemmed != w {
                // Search both the stem (prefix) and the original (prefix)
                format!("{}* OR {}*", stemmed, w)
            } else {
                format!("{}*", w)
            }
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Lightweight English stemmer for FTS query improvement.
/// Strips common suffixes so "hackathons" -> "hackathon", "running" -> "run", etc.
/// Not a full Porter stemmer — just handles the most common cases that cause
/// FTS prefix-match failures when the query word is longer than the stored word.
pub fn simple_stem(word: &str) -> String {
    let w = word.to_lowercase();

    // Order matters: try longest suffixes first
    if let Some(base) = w.strip_suffix("ings") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ing") {
        if base.len() >= 3 {
            // running -> run (double consonant)
            let bytes = base.as_bytes();
            if bytes.len() >= 2 && bytes[bytes.len() - 1] == bytes[bytes.len() - 2] {
                return base[..base.len() - 1].to_string();
            }
            return base.to_string();
        }
    }
    if let Some(base) = w.strip_suffix("ations") {
        if base.len() >= 2 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ation") {
        if base.len() >= 2 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ness") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ments") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ment") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ies") {
        if base.len() >= 2 { return format!("{}y", base); }
    }
    if let Some(base) = w.strip_suffix("edly") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ed") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ers") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("er") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ions") {
        if base.len() >= 3 { return base.to_string(); }
    }
    if let Some(base) = w.strip_suffix("ion") {
        if base.len() >= 3 { return base.to_string(); }
    }
    // Simple plural: "hackathons" -> "hackathon", but not "ss" words like "class"
    if let Some(base) = w.strip_suffix('s') {
        if base.len() >= 3 && !base.ends_with('s') {
            return base.to_string();
        }
    }

    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_stem_plurals() {
        assert_eq!(simple_stem("hackathons"), "hackathon");
        assert_eq!(simple_stem("events"), "event");
        // "class" should NOT be stemmed (ends in ss)
        assert_eq!(simple_stem("class"), "class");
    }

    #[test]
    fn test_simple_stem_ing() {
        assert_eq!(simple_stem("running"), "run");
        assert_eq!(simple_stem("building"), "build");
    }

    #[test]
    fn test_normalize_fts_query_basic() {
        let result = normalize_fts_query("hackathons");
        assert!(result.contains("hackathon*"), "should contain stemmed prefix: {}", result);
        assert!(result.contains("hackathons*"), "should contain original prefix: {}", result);
    }

    #[test]
    fn test_normalize_fts_query_filters_stop_words() {
        let result = normalize_fts_query("the quick brown fox");
        // "the" is a stop word
        assert!(!result.contains("the"), "should not contain stop word 'the': {}", result);
        assert!(result.contains("quick*"), "should contain 'quick': {}", result);
    }

    #[test]
    fn test_normalize_fts_query_empty() {
        assert_eq!(normalize_fts_query(""), "");
        assert_eq!(normalize_fts_query("   "), "");
        // Only stop words
        assert_eq!(normalize_fts_query("the a an"), "");
    }

    #[test]
    fn test_normalize_fts_query_limits_to_8_words() {
        let long_query = "alpha bravo charlie delta echo foxtrot golf hotel india juliet";
        let result = normalize_fts_query(long_query);
        // Count the number of distinct stem groups (separated by " OR " at the top level)
        // Each word produces either "stem* OR word*" or "word*"
        // With 8 words that don't stem, we get 8 "word*" entries joined by " OR "
        let parts: Vec<&str> = result.split(" OR ").collect();
        assert!(parts.len() <= 8, "should have at most 8 token groups, got {}: {}", parts.len(), result);
    }
}
