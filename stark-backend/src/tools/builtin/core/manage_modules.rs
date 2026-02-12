//! Module management tool — install, uninstall, enable, disable, list, and check status of plugins

use crate::tools::registry::Tool;
use crate::tools::types::{
    PropertySchema, ToolContext, ToolDefinition, ToolGroup, ToolInputSchema, ToolResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ManageModulesTool {
    definition: ToolDefinition,
}

impl ManageModulesTool {
    pub fn new() -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "action".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Action: 'list' available modules, 'install' a module, 'uninstall', 'enable', 'disable', or check 'status'".to_string(),
                default: None,
                items: None,
                enum_values: Some(vec![
                    "list".to_string(),
                    "install".to_string(),
                    "uninstall".to_string(),
                    "enable".to_string(),
                    "disable".to_string(),
                    "status".to_string(),
                ]),
            },
        );

        properties.insert(
            "name".to_string(),
            PropertySchema {
                schema_type: "string".to_string(),
                description: "Module name (required for install, uninstall, enable, disable, status)".to_string(),
                default: None,
                items: None,
                enum_values: None,
            },
        );

        ManageModulesTool {
            definition: ToolDefinition {
                name: "manage_modules".to_string(),
                description: "Manage StarkBot plugin modules. List available modules, install/uninstall features, enable/disable, or check status. Modules add optional features like wallet monitoring.".to_string(),
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

#[derive(Debug, Deserialize)]
struct ModuleParams {
    action: String,
    name: Option<String>,
}

#[async_trait]
impl Tool for ManageModulesTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
        let params: ModuleParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let db = match context.db.as_ref() {
            Some(db) => db,
            None => return ToolResult::error("Database not available"),
        };

        match params.action.as_str() {
            "list" => {
                // Use the compile-time module registry
                let registry = crate::modules::ModuleRegistry::new();
                let installed = db.list_installed_modules().unwrap_or_default();

                let mut output = String::from("**Available Modules**\n\n");

                for module in registry.available_modules() {
                    let installed_entry = installed.iter().find(|m| m.module_name == module.name());
                    let status = match installed_entry {
                        Some(e) if e.enabled => "installed & enabled",
                        Some(_) => "installed (disabled)",
                        None => "not installed",
                    };

                    // Check if required API keys are present
                    let required_keys = module.required_api_keys();
                    let keys_met = required_keys.iter().all(|key| {
                        db.get_api_key(key).ok().flatten().is_some()
                    });
                    let keys_status = if required_keys.is_empty() {
                        "none required".to_string()
                    } else if keys_met {
                        format!("met ({})", required_keys.join(", "))
                    } else {
                        format!("MISSING ({})", required_keys.join(", "))
                    };

                    output.push_str(&format!(
                        "**{}** v{} — {}\n  Status: {} | API Keys: {} | Tables: {} | Tools: {} | Worker: {}\n\n",
                        module.name(),
                        module.version(),
                        module.description(),
                        status,
                        keys_status,
                        if module.has_db_tables() { "yes" } else { "no" },
                        if module.has_tools() { "yes" } else { "no" },
                        if module.has_worker() { "yes" } else { "no" },
                    ));
                }

                ToolResult::success(output)
            }

            "install" => {
                let name = match params.name.as_deref() {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for 'install' action"),
                };

                // Check if already installed
                if db.is_module_installed(name).unwrap_or(false) {
                    return ToolResult::error(format!("Module '{}' is already installed. Use 'enable' to re-enable it.", name));
                }

                // Look up in compile-time registry
                let registry = crate::modules::ModuleRegistry::new();
                let module = match registry.get(name) {
                    Some(m) => m,
                    None => return ToolResult::error(format!("Unknown module: '{}'. Use action='list' to see available modules.", name)),
                };

                // Check API key dependencies
                for key in module.required_api_keys() {
                    if db.get_api_key(key).ok().flatten().is_none() {
                        return ToolResult::error(format!(
                            "Module '{}' requires API key '{}' which is not configured. Install it first with install_api_key.",
                            name, key
                        ));
                    }
                }

                // Create DB tables if needed
                if module.has_db_tables() {
                    let conn = db.conn();
                    if let Err(e) = module.init_tables(&conn) {
                        return ToolResult::error(format!("Failed to create tables for module '{}': {}", name, e));
                    }
                }

                // Install module record
                let required_keys = module.required_api_keys();
                let key_strs: Vec<&str> = required_keys.iter().copied().collect();
                match db.install_module(
                    name,
                    module.description(),
                    module.version(),
                    module.has_db_tables(),
                    module.has_tools(),
                    module.has_worker(),
                    &key_strs,
                ) {
                    Ok(_entry) => {
                        let mut result_parts = vec![format!("Module '{}' installed successfully!", name)];

                        if module.has_db_tables() {
                            result_parts.push("Database tables created.".to_string());
                        }
                        if module.has_tools() {
                            result_parts.push("Tools registered (available after restart or on next session).".to_string());
                        }
                        if module.has_worker() {
                            result_parts.push("Background worker will start on next restart.".to_string());
                        }

                        // Install skill if module provides one
                        if let Some(skill_md) = module.skill_content() {
                            if let Some(skill_registry) = context.skill_registry.as_ref() {
                                match skill_registry.create_skill_from_markdown(skill_md, None).await {
                                    Ok(_) => result_parts.push("Skill installed.".to_string()),
                                    Err(e) => result_parts.push(format!("Warning: Failed to install skill: {}", e)),
                                }
                            }
                        }

                        ToolResult::success(result_parts.join("\n"))
                    }
                    Err(e) => ToolResult::error(format!("Failed to install module: {}", e)),
                }
            }

            "uninstall" => {
                let name = match params.name.as_deref() {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for 'uninstall' action"),
                };
                match db.uninstall_module(name) {
                    Ok(true) => ToolResult::success(format!(
                        "Module '{}' uninstalled. Data tables are preserved. Restart to fully remove tools and worker.",
                        name
                    )),
                    Ok(false) => ToolResult::error(format!("Module '{}' is not installed", name)),
                    Err(e) => ToolResult::error(format!("Failed to uninstall: {}", e)),
                }
            }

            "enable" => {
                let name = match params.name.as_deref() {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for 'enable' action"),
                };
                match db.set_module_enabled(name, true) {
                    Ok(true) => ToolResult::success(format!(
                        "Module '{}' enabled. Worker will resume on next restart.",
                        name
                    )),
                    Ok(false) => ToolResult::error(format!("Module '{}' is not installed", name)),
                    Err(e) => ToolResult::error(format!("Failed to enable: {}", e)),
                }
            }

            "disable" => {
                let name = match params.name.as_deref() {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for 'disable' action"),
                };
                match db.set_module_enabled(name, false) {
                    Ok(true) => ToolResult::success(format!(
                        "Module '{}' disabled. Worker will stop, tools hidden. Data preserved.",
                        name
                    )),
                    Ok(false) => ToolResult::error(format!("Module '{}' is not installed", name)),
                    Err(e) => ToolResult::error(format!("Failed to disable: {}", e)),
                }
            }

            "status" => {
                let name = match params.name.as_deref() {
                    Some(n) => n,
                    None => return ToolResult::error("'name' is required for 'status' action"),
                };
                match db.get_installed_module(name) {
                    Ok(Some(m)) => {
                        let keys_met = m.required_api_keys.iter().all(|key| {
                            db.get_api_key(key).ok().flatten().is_some()
                        });
                        ToolResult::success(json!({
                            "module": m.module_name,
                            "version": m.version,
                            "enabled": m.enabled,
                            "description": m.description,
                            "has_db_tables": m.has_db_tables,
                            "has_tools": m.has_tools,
                            "has_worker": m.has_worker,
                            "api_keys_met": keys_met,
                            "required_api_keys": m.required_api_keys,
                            "installed_at": m.installed_at.to_rfc3339(),
                        }).to_string())
                    }
                    Ok(None) => ToolResult::error(format!("Module '{}' is not installed", name)),
                    Err(e) => ToolResult::error(format!("Failed to get status: {}", e)),
                }
            }

            _ => ToolResult::error(format!(
                "Unknown action: '{}'. Use 'list', 'install', 'uninstall', 'enable', 'disable', or 'status'.",
                params.action
            )),
        }
    }

    fn safety_level(&self) -> crate::tools::types::ToolSafetyLevel {
        crate::tools::types::ToolSafetyLevel::Standard
    }
}
