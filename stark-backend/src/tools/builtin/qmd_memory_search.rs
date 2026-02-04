//! QMD Memory Search Tool
//!
//! Full-text search across all memory markdown files using FTS5 BM25 ranking.

use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool for searching memories using full-text search
pub struct QmdMemorySearchTool {
    definition: ToolDefinition,
}

impl QmdMemorySearchTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "query".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Search query - words to search for in memories. Multiple words are matched with OR logic.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "limit".to_string(),
            PropertySchema {
                schema_type: "integer".to_string(),
                description: "Maximum number of results to return (default: 10, max: 50).".to_string(),
                default: Some(json!(10)),
                items: None,
                enum_values: None,
            },
        );

        Self {
            definition: ToolDefinition {
                name: "memory_search".to_string(),
                description: "Search across all memory files for relevant information. Returns ranked results with file paths and matching snippets. Use this to find past conversations, facts, preferences, or any stored knowledge.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["query".to_string()],
                },
                group: ToolGroup::System,
            },
        }
    }
}

impl Default for QmdMemorySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    query: String,
    limit: Option<i32>,
}

#[async_trait]
impl Tool for QmdMemorySearchTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: SearchParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Validate query
        if params.query.trim().is_empty() {
            return ToolResult::error("Query cannot be empty");
        }

        // Get memory store from context
        let memory_store = match &context.memory_store {
            Some(store) => store,
            None => {
                return ToolResult::error(
                    "Memory store not available. Memory search requires the memory system to be initialized.",
                );
            }
        };

        let limit = params.limit.unwrap_or(10).min(50).max(1);

        // Perform search
        match memory_store.search(&params.query, limit) {
            Ok(results) => {
                if results.is_empty() {
                    return ToolResult::success(format!(
                        "No memories found matching: \"{}\"",
                        params.query
                    ));
                }

                let mut output = format!(
                    "## Memory Search Results\n**Query:** \"{}\"\n**Found:** {} result(s)\n\n",
                    params.query,
                    results.len()
                );

                for (i, result) in results.iter().enumerate() {
                    output.push_str(&format!(
                        "### {}. {}\n**Score:** {:.2}\n{}\n\n",
                        i + 1,
                        result.file_path,
                        -result.score, // Negate because BM25 returns negative scores
                        result.snippet.replace(">>>", "**").replace("<<<", "**")
                    ));
                }

                ToolResult::success(output).with_metadata(json!({
                    "query": params.query,
                    "result_count": results.len(),
                    "files": results.iter().map(|r| r.file_path.clone()).collect::<Vec<_>>()
                }))
            }
            Err(e) => ToolResult::error(format!("Search failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_search_definition() {
        let tool = QmdMemorySearchTool::new();
        let def = tool.definition();

        assert_eq!(def.name, "memory_search");
        assert_eq!(def.group, ToolGroup::System);
        assert!(def.input_schema.required.contains(&"query".to_string()));
    }
}
