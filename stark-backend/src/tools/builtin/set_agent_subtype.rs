use crate::ai::multi_agent::types::AgentSubtype;
use crate::gateway::protocol::GatewayEvent;
use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool to switch between agent subtypes (Finance, CodeEngineer, Secretary)
/// This controls which tools and skills are available to the agent.
/// Think of subtypes as "toolboxes" - each one unlocks different capabilities.
pub struct SetAgentSubtypeTool {
    definition: ToolDefinition,
}

impl SetAgentSubtypeTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "subtype".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "The agent subtype/toolbox to switch to:\n\
                    • 'finance' - DeFi/crypto operations (swaps, transfers, web3)\n\
                    • 'code_engineer' - Software development (code editing, git, testing)\n\
                    • 'secretary' - Social media, messaging, scheduling, marketing".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "finance".to_string(),
                    "code_engineer".to_string(),
                    "secretary".to_string(),
                ]),
            },
        );

        SetAgentSubtypeTool {
            definition: ToolDefinition {
                name: "set_agent_subtype".to_string(),
                description: "Switch the agent's toolbox/specialization. Each subtype unlocks different tools and skills:\n\
                    • 'finance' - Crypto swaps, transfers, token lookups, DeFi operations\n\
                    • 'code_engineer' - Code editing, git, grep/glob search, testing, debugging\n\
                    • 'secretary' - Social media posting, messaging, scheduling, marketing\n\
                    Core tools (read_file, list_files, web_fetch) are always available.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["subtype".to_string()],
                },
                group: ToolGroup::System,
            },
        }
    }

    /// Get a description of available tools for a subtype
    fn describe_subtype(subtype: AgentSubtype) -> String {
        match subtype {
            AgentSubtype::Finance => {
                "💰 Finance toolbox activated.\n\n\
                 Tools now available:\n\
                 • web3_tx - Execute blockchain transactions\n\
                 • web3_function_call - Read smart contract data (use presets like erc20_balance)\n\
                 • token_lookup - Get token info and addresses\n\
                 • x402_rpc - RPC calls (get_balance, gas_price, etc.)\n\
                 • x402_fetch - Payment protocol fetch operations\n\
                 • register_set - Store transaction data safely\n\
                 • ask_user - Ask user for clarification (e.g., which network)\n\n\
                 Note: wallet_address is an intrinsic register - always available.\n\n\
                 Skills: swap, transfer, bankr, token_price, weth, local_wallet"
                    .to_string()
            }
            AgentSubtype::CodeEngineer => {
                "🛠️ CodeEngineer toolbox activated.\n\n\
                 Tools now available:\n\
                 • grep - Search file contents with regex\n\
                 • glob - Find files by pattern\n\
                 • edit_file - Precise string replacement\n\
                 • write_file - Create/overwrite files\n\
                 • delete_file - Remove files/directories\n\
                 • rename_file - Move/rename files\n\
                 • git - Git operations (status, diff, commit, branch)\n\
                 • exec - Run shell commands\n\n\
                 Skills: plan, commit, test, debug, code-review, github"
                    .to_string()
            }
            AgentSubtype::Secretary => {
                "📱 Secretary toolbox activated.\n\n\
                 Tools now available:\n\
                 • agent_send - Send messages to other channels\n\
                 • (Social tools for Twitter, scheduling coming soon)\n\n\
                 Skills: send_tweet, twitter, scheduling"
                    .to_string()
            }
        }
    }
}

impl Default for SetAgentSubtypeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct SetAgentSubtypeParams {
    subtype: String,
}

#[async_trait]
impl Tool for SetAgentSubtypeTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: SetAgentSubtypeParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Parse the subtype
        let subtype = match AgentSubtype::from_str(&params.subtype) {
            Some(s) => s,
            None => {
                return ToolResult::error(format!(
                    "Invalid subtype '{}'. Valid options: 'finance', 'code_engineer', 'secretary'",
                    params.subtype
                ))
            }
        };

        // Broadcast the subtype change event
        if let (Some(broadcaster), Some(channel_id)) = (&context.broadcaster, context.channel_id) {
            broadcaster.broadcast(GatewayEvent::agent_subtype_change(
                channel_id,
                subtype.as_str(),
                subtype.label(),
            ));
        }

        // Return success with description of available tools
        let description = Self::describe_subtype(subtype);
        ToolResult::success(description).with_metadata(json!({
            "subtype": subtype.as_str(),
            "label": subtype.label(),
            "emoji": subtype.emoji(),
            "allowed_tool_groups": subtype.allowed_tool_groups()
                .iter()
                .map(|g| g.as_str())
                .collect::<Vec<_>>(),
            "allowed_skill_tags": subtype.allowed_skill_tags(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_subtype_finance() {
        let tool = SetAgentSubtypeTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({ "subtype": "finance" }), &context)
            .await;

        assert!(result.success);
        assert!(result.content.contains("Finance toolbox"));
    }

    #[tokio::test]
    async fn test_set_subtype_code_engineer() {
        let tool = SetAgentSubtypeTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({ "subtype": "code_engineer" }), &context)
            .await;

        assert!(result.success);
        assert!(result.content.contains("CodeEngineer toolbox"));
    }

    #[tokio::test]
    async fn test_set_subtype_secretary() {
        let tool = SetAgentSubtypeTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({ "subtype": "secretary" }), &context)
            .await;

        assert!(result.success);
        assert!(result.content.contains("Secretary toolbox"));
    }

    #[tokio::test]
    async fn test_invalid_subtype() {
        let tool = SetAgentSubtypeTool::new();
        let context = ToolContext::new();

        let result = tool
            .execute(json!({ "subtype": "invalid" }), &context)
            .await;

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid subtype"));
    }
}
