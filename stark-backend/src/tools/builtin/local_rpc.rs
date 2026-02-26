//! Local RPC tool — calls module services by name via the port registry.
//!
//! Agents use `local_rpc(module="spot_trader", path="/rpc/decision", ...)`
//! and the tool resolves the module name to `http://127.0.0.1:<port>` via
//! the port registry.  No hardcoded ports needed in templates.

use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
    ToolSafetyLevel,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct LocalRpcTool {
    definition: ToolDefinition,
}

impl LocalRpcTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "module".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Module name (e.g. \"spot_trader\", \"wallet_monitor\"). Resolved to localhost:<port> via the port registry.".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "path".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Request path (e.g. \"/rpc/decision\", \"/rpc/status\")".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        properties.insert(
            "method".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "HTTP method (default: GET)".to_string(),
                default: Some(json!("GET")),
                items: None,
                enum_values: Some(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                ]),
            },
        );

        properties.insert(
            "body".to_string(),
            PropertySchema {
                schema_type: "object".to_string(),
                description: "JSON request body (for POST/PUT/PATCH)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        LocalRpcTool {
            definition: ToolDefinition {
                name: "local_rpc".to_string(),
                description: "Call a module's local RPC endpoint. Specify the module name and path — the port is resolved automatically.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["module".to_string(), "path".to_string()],
                },
                group: ToolGroup::System,
                hidden: false,
            },
        }
    }
}

impl Default for LocalRpcTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct LocalRpcParams {
    module: String,
    path: String,
    method: Option<String>,
    body: Option<Value>,
}

#[async_trait]
impl Tool for LocalRpcTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, _context: &ToolContext) -> ToolResult {
        let params: LocalRpcParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Resolve module name to port
        let port = match crate::modules::port_registry::resolve(&params.module) {
            Some(p) => p,
            None => {
                return ToolResult::error(format!(
                    "Unknown module '{}'. Is it installed and running?",
                    params.module
                ));
            }
        };

        // Build URL: http://127.0.0.1:<port><path>
        let path = if params.path.starts_with('/') {
            params.path.clone()
        } else {
            format!("/{}", params.path)
        };
        let url = format!("http://127.0.0.1:{}{}", port, path);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let method = params.method.as_deref().unwrap_or("GET").to_uppercase();

        let mut request = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };

        // Default to JSON content type for all requests
        request = request.header("Content-Type", "application/json");

        // Attach JSON body for write methods
        if let Some(ref body) = params.body {
            if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
                request = request.body(
                    serde_json::to_string(body)
                        .unwrap_or_else(|_| body.to_string()),
                );
            }
        }

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                return ToolResult::error(format!(
                    "Request to {} failed: {}",
                    params.module, e
                ));
            }
        };

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            let truncated = if body.len() > 2000 {
                format!("{}...", &body[..2000])
            } else {
                body
            };
            return ToolResult::error(format!(
                "HTTP {} from {}:{}\n{}",
                status, params.module, path, truncated
            ));
        }

        ToolResult::success(body)
    }

    fn safety_level(&self) -> ToolSafetyLevel {
        ToolSafetyLevel::Standard
    }
}
