//! Set NFT Token ID tool — typed uint256 register setter
//!
//! Sets the `nft_token_id` register after validating that the value
//! is a valid non-negative integer string (uint256-compatible).

use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Set NFT Token ID tool — validates and stores a token ID in the nft_token_id register
pub struct SetNftTokenIdTool {
    definition: ToolDefinition,
}

impl SetNftTokenIdTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "token_id".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "The NFT token ID (non-negative integer as a string).".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        SetNftTokenIdTool {
            definition: ToolDefinition {
                name: "set_nft_token_id".to_string(),
                description: "Set an NFT token ID in the 'nft_token_id' register. Validates that the value is a non-negative integer. Use this before calling NFT presets that require a token ID (e.g. nft_owner_of, nft_safe_transfer_from, nft_token_uri).".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["token_id".to_string()],
                },
                group: ToolGroup::Finance,
                hidden: false,
            },
        }
    }

    /// Validate that the string represents a non-negative integer (uint256-compatible)
    fn is_valid_token_id(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        // Must be all digits
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        // No leading zeros (except "0" itself)
        if s.len() > 1 && s.starts_with('0') {
            return false;
        }
        true
    }
}

impl Default for SetNftTokenIdTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SetNftTokenIdParams {
    token_id: String,
}

#[async_trait]
impl Tool for SetNftTokenIdTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: SetNftTokenIdParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Validate token ID format
        if !Self::is_valid_token_id(&params.token_id) {
            return ToolResult::error(format!(
                "Invalid token ID '{}'. Must be a non-negative integer (e.g. \"0\", \"42\", \"1234\").",
                params.token_id
            ));
        }

        // Store in register
        context.set_register("nft_token_id", json!(&params.token_id), "set_nft_token_id");

        log::info!(
            "[set_nft_token_id] Set register 'nft_token_id' = '{}'",
            params.token_id
        );

        ToolResult::success(format!("Set 'nft_token_id' = {}", params.token_id))
            .with_metadata(json!({
                "register": "nft_token_id",
                "token_id": params.token_id
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_token_ids() {
        assert!(SetNftTokenIdTool::is_valid_token_id("0"));
        assert!(SetNftTokenIdTool::is_valid_token_id("1"));
        assert!(SetNftTokenIdTool::is_valid_token_id("42"));
        assert!(SetNftTokenIdTool::is_valid_token_id("1234567890"));
        assert!(SetNftTokenIdTool::is_valid_token_id(
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        ));
    }

    #[test]
    fn test_invalid_token_ids() {
        // Empty
        assert!(!SetNftTokenIdTool::is_valid_token_id(""));
        // Negative
        assert!(!SetNftTokenIdTool::is_valid_token_id("-1"));
        // Decimal
        assert!(!SetNftTokenIdTool::is_valid_token_id("1.5"));
        // Hex
        assert!(!SetNftTokenIdTool::is_valid_token_id("0x1"));
        // Leading zeros
        assert!(!SetNftTokenIdTool::is_valid_token_id("01"));
        assert!(!SetNftTokenIdTool::is_valid_token_id("007"));
        // Letters
        assert!(!SetNftTokenIdTool::is_valid_token_id("abc"));
        // Spaces
        assert!(!SetNftTokenIdTool::is_valid_token_id("1 2"));
    }

    #[tokio::test]
    async fn test_set_valid_token_id() {
        let tool = SetNftTokenIdTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({"token_id": "42"}), &context)
            .await;

        assert!(result.success, "Should accept valid token ID");
        assert!(result.content.contains("42"));
    }

    #[tokio::test]
    async fn test_set_zero_token_id() {
        let tool = SetNftTokenIdTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({"token_id": "0"}), &context)
            .await;

        assert!(result.success, "Should accept token ID 0");
    }

    #[tokio::test]
    async fn test_reject_invalid_token_id() {
        let tool = SetNftTokenIdTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({"token_id": "-1"}), &context)
            .await;

        assert!(!result.success, "Should reject negative token ID");
        assert!(result.content.contains("Invalid token ID"));
    }

    #[tokio::test]
    async fn test_reject_leading_zeros() {
        let tool = SetNftTokenIdTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({"token_id": "007"}), &context)
            .await;

        assert!(!result.success, "Should reject leading zeros");
    }
}
