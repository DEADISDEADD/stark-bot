use crate::eip8004::types::{RegistrationFile, ServiceEntry};
use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool for the agent to manage its EIP-8004 identity registration (stored in DB)
pub struct ModifyIdentityTool {
    definition: ToolDefinition,
}

impl ModifyIdentityTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "action".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Action: 'read' view identity, 'create' new identity, 'update_field' update a field, 'add_service' add service entry, 'remove_service' remove service entry, 'upload' publish to identity.defirelay.com".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "read".to_string(),
                    "create".to_string(),
                    "update_field".to_string(),
                    "add_service".to_string(),
                    "remove_service".to_string(),
                    "upload".to_string(),
                ]),
            },
        );
        properties.insert(
            "name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Agent name (for create action)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "description".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Agent description (for create action)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "image".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Image URL (for create/update_field)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "field".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Field to update: name, description, image, active (for update_field)".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "name".to_string(),
                    "description".to_string(),
                    "image".to_string(),
                    "active".to_string(),
                ]),
            },
        );
        properties.insert(
            "value".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "New value for the field (for update_field)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "service_name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Service name, e.g. 'mcp', 'a2a', 'chat', 'x402', 'swap' (for add_service/remove_service)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "service_endpoint".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Service endpoint URL (for add_service)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );
        properties.insert(
            "service_version".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Service version (for add_service, default '1.0')".to_string(),
                default: Some(serde_json::Value::String("1.0".to_string())),
                items: None,
                enum_values: None,
            },
        );

        ModifyIdentityTool {
            definition: ToolDefinition {
                name: "modify_identity".to_string(),
                description: "Manage your EIP-8004 agent identity (stored in DB). Create, read, update fields, add/remove services, or upload to identity.defirelay.com.".to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties,
                    required: vec!["action".to_string()],
                },
                group: ToolGroup::System,
                hidden: false,
            },
        }
    }
}

impl Default for ModifyIdentityTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ModifyIdentityParams {
    action: String,
    name: Option<String>,
    description: Option<String>,
    image: Option<String>,
    field: Option<String>,
    value: Option<String>,
    service_name: Option<String>,
    service_endpoint: Option<String>,
    service_version: Option<String>,
}

#[async_trait]
impl Tool for ModifyIdentityTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: ModifyIdentityParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let db = match &context.database {
            Some(db) => db,
            None => return ToolResult::error("Database not available"),
        };

        match params.action.as_str() {
            "read" => {
                match db.get_agent_identity_full() {
                    Some(row) => {
                        let reg = row.to_registration_file();
                        let json_str = serde_json::to_string_pretty(&reg).unwrap_or_default();
                        ToolResult::success(format!(
                            "=== Agent Identity (DB) ===\n\
                            Agent ID: {}\n\
                            Registry: {}\n\
                            Chain ID: {}\n\
                            Registration URI: {}\n\n\
                            === Metadata ===\n{}",
                            row.agent_id, row.agent_registry, row.chain_id,
                            row.registration_uri.as_deref().unwrap_or("(none)"),
                            json_str
                        )).with_metadata(json!({
                            "action": "read",
                            "agent_id": row.agent_id,
                            "agent_registry": row.agent_registry,
                            "chain_id": row.chain_id,
                            "name": row.name,
                            "registered": true,
                        }))
                    }
                    None => {
                        ToolResult::success(
                            "No identity found in database.\n\n\
                            Use action='create' to create a new identity, or import_identity to import an existing on-chain NFT."
                        )
                    }
                }
            }

            "create" => {
                // Refuse to overwrite an existing identity
                if db.get_agent_identity_full().is_some() {
                    return ToolResult::error(
                        "Identity already exists in the database. Use 'update_field' to modify it."
                    );
                }

                let name = match params.name {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for create action"),
                };
                let description = match params.description {
                    Some(d) => d,
                    None => return ToolResult::error("'description' is required for create action"),
                };

                let reg = RegistrationFile::new(&name, &description);
                let services_json = serde_json::to_string(&reg.services).unwrap_or_else(|_| "[]".to_string());
                let supported_trust_json = serde_json::to_string(&reg.supported_trust).unwrap_or_else(|_| "[]".to_string());

                // Create with agent_id=0 (not yet registered on-chain)
                match db.upsert_agent_identity(
                    0, "", 0,
                    Some(&name), Some(&description), params.image.as_deref(),
                    true, true,
                    &services_json, &supported_trust_json,
                    None,
                ) {
                    Ok(_) => {
                        let mut created_reg = reg;
                        if let Some(ref img) = params.image {
                            created_reg.image = Some(img.clone());
                        }
                        let json = serde_json::to_string_pretty(&created_reg).unwrap_or_default();
                        log::info!("Created agent identity in DB for agent: {}", name);
                        ToolResult::success(format!("Identity created successfully:\n{}", json))
                            .with_metadata(json!({
                                "action": "create",
                                "name": name
                            }))
                    }
                    Err(e) => ToolResult::error(format!("Failed to create identity: {}", e)),
                }
            }

            "update_field" => {
                let field = match params.field {
                    Some(f) => f,
                    None => return ToolResult::error("'field' is required for update_field action"),
                };
                let value = match params.value {
                    Some(v) => v,
                    None => return ToolResult::error("'value' is required for update_field action"),
                };

                if db.get_agent_identity_full().is_none() {
                    return ToolResult::error("No identity found. Use action='create' first.");
                }

                // For 'active' field, convert to integer
                let db_value = match field.as_str() {
                    "active" => if value.to_lowercase() == "true" { "1".to_string() } else { "0".to_string() },
                    "name" | "description" | "image" => value.clone(),
                    _ => return ToolResult::error(format!("Unknown field '{}'. Valid: name, description, image, active", field)),
                };

                match db.update_agent_identity_field(&field, &db_value) {
                    Ok(_) => {
                        log::info!("Updated agent identity field '{}' to '{}'", field, value);
                        ToolResult::success(format!("Updated '{}' to '{}'", field, value))
                            .with_metadata(json!({
                                "action": "update_field",
                                "field": field,
                                "value": value
                            }))
                    }
                    Err(e) => ToolResult::error(format!("Failed to update field: {}", e)),
                }
            }

            "add_service" => {
                let service_name = match params.service_name {
                    Some(n) => n,
                    None => return ToolResult::error("'service_name' is required for add_service action"),
                };
                let endpoint = match params.service_endpoint {
                    Some(e) => e,
                    None => return ToolResult::error("'service_endpoint' is required for add_service action"),
                };
                let version = params.service_version.unwrap_or_else(|| "1.0".to_string());

                let row = match db.get_agent_identity_full() {
                    Some(r) => r,
                    None => return ToolResult::error("No identity found. Use action='create' first."),
                };

                let mut services: Vec<ServiceEntry> =
                    serde_json::from_str(&row.services_json).unwrap_or_default();
                services.push(ServiceEntry {
                    name: service_name.clone(),
                    endpoint: endpoint.clone(),
                    version: version.clone(),
                });

                let new_json = serde_json::to_string(&services).unwrap_or_else(|_| "[]".to_string());
                match db.update_agent_identity_field("services_json", &new_json) {
                    Ok(_) => {
                        log::info!("Added service '{}' to agent identity", service_name);
                        ToolResult::success(format!("Added service '{}' at {}", service_name, endpoint))
                            .with_metadata(json!({
                                "action": "add_service",
                                "service_name": service_name,
                                "endpoint": endpoint,
                                "version": version
                            }))
                    }
                    Err(e) => ToolResult::error(format!("Failed to add service: {}", e)),
                }
            }

            "remove_service" => {
                let service_name = match params.service_name {
                    Some(n) => n,
                    None => return ToolResult::error("'service_name' is required for remove_service action"),
                };

                let row = match db.get_agent_identity_full() {
                    Some(r) => r,
                    None => return ToolResult::error("No identity found. Use action='create' first."),
                };

                let mut services: Vec<ServiceEntry> =
                    serde_json::from_str(&row.services_json).unwrap_or_default();
                let before = services.len();
                services.retain(|s| s.name != service_name);
                let removed = before - services.len();

                if removed == 0 {
                    return ToolResult::error(format!("Service '{}' not found in identity", service_name));
                }

                let new_json = serde_json::to_string(&services).unwrap_or_else(|_| "[]".to_string());
                match db.update_agent_identity_field("services_json", &new_json) {
                    Ok(_) => {
                        log::info!("Removed service '{}' from agent identity", service_name);
                        ToolResult::success(format!("Removed service '{}'", service_name))
                            .with_metadata(json!({
                                "action": "remove_service",
                                "service_name": service_name,
                                "removed_count": removed
                            }))
                    }
                    Err(e) => ToolResult::error(format!("Failed to remove service: {}", e)),
                }
            }

            "upload" => {
                let row = match db.get_agent_identity_full() {
                    Some(r) => r,
                    None => return ToolResult::error("No identity found. Create one first with action='create'."),
                };

                // Serialize from DB row to RegistrationFile JSON
                let reg = row.to_registration_file();
                let json_content = match serde_json::to_string_pretty(&reg) {
                    Ok(j) => j,
                    Err(e) => return ToolResult::error(format!("Failed to serialize identity: {}", e)),
                };

                // Use the identity client to upload
                use crate::identity_client::IDENTITY_CLIENT;

                // Use wallet provider (Privy/Flash) for SIWE authentication
                let wallet_provider = match &context.wallet_provider {
                    Some(wp) => wp,
                    None => return ToolResult::error(
                        "No wallet connected. Connect your wallet first to upload your identity."
                    ),
                };
                let upload_result = IDENTITY_CLIENT
                    .upload_identity_with_provider(wallet_provider, &json_content)
                    .await;

                match upload_result {
                    Ok(resp) => {
                        if resp.success {
                            let url = resp.url.unwrap_or_else(|| "unknown".to_string());
                            log::info!("Uploaded identity to {}", url);

                            // Store registration_uri in DB
                            let _ = db.update_agent_identity_field("registration_uri", &url);

                            // Set the agent_uri register so the identity_register preset can use it
                            context.set_register("agent_uri", json!(&url), "modify_identity");

                            ToolResult::success(format!("Identity uploaded successfully!\nHosted at: {}\n\nThe agent_uri register has been set — you can now call identity_register.", url))
                                .with_metadata(json!({
                                    "action": "upload",
                                    "url": url,
                                    "success": true
                                }))
                        } else {
                            let error = resp.error.unwrap_or_else(|| "Unknown error".to_string());
                            ToolResult::error(format!("Upload failed: {}. STOP — do not proceed with on-chain registration until the upload succeeds.", error))
                        }
                    }
                    Err(e) => ToolResult::error(format!("Upload failed: {}. STOP — do not proceed with on-chain registration until the upload succeeds.", e)),
                }
            }

            _ => ToolResult::error(format!(
                "Unknown action: '{}'. Use: read, create, update_field, add_service, remove_service, upload",
                params.action
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_creation() {
        let tool = ModifyIdentityTool::new();
        assert_eq!(tool.definition().name, "modify_identity");
        assert_eq!(tool.definition().group, ToolGroup::System);
    }
}
