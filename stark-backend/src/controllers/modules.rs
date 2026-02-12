//! HTTP API endpoints for the module/plugin system

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Serialize)]
struct ModuleInfo {
    name: String,
    description: String,
    version: String,
    installed: bool,
    enabled: bool,
    has_db_tables: bool,
    has_tools: bool,
    has_worker: bool,
    required_api_keys: Vec<String>,
    api_keys_met: bool,
    installed_at: Option<String>,
}

#[derive(Deserialize)]
struct ModuleActionRequest {
    action: String, // "install", "uninstall", "enable", "disable"
}

/// GET /api/modules — list all available modules with install status
async fn list_modules(data: web::Data<AppState>) -> HttpResponse {
    let registry = crate::modules::ModuleRegistry::new();
    let installed = data.db.list_installed_modules().unwrap_or_default();

    let mut modules = Vec::new();
    for module in registry.available_modules() {
        let installed_entry = installed.iter().find(|m| m.module_name == module.name());
        let required_keys: Vec<String> = module.required_api_keys().iter().map(|s| s.to_string()).collect();
        let api_keys_met = required_keys.iter().all(|key| {
            data.db.get_api_key(key).ok().flatten().is_some()
        });

        modules.push(ModuleInfo {
            name: module.name().to_string(),
            description: module.description().to_string(),
            version: module.version().to_string(),
            installed: installed_entry.is_some(),
            enabled: installed_entry.map(|e| e.enabled).unwrap_or(false),
            has_db_tables: module.has_db_tables(),
            has_tools: module.has_tools(),
            has_worker: module.has_worker(),
            required_api_keys: required_keys,
            api_keys_met,
            installed_at: installed_entry.map(|e| e.installed_at.to_rfc3339()),
        });
    }

    HttpResponse::Ok().json(modules)
}

/// POST /api/modules/{name} — install, uninstall, enable, or disable a module
async fn module_action(
    data: web::Data<AppState>,
    name: web::Path<String>,
    body: web::Json<ModuleActionRequest>,
) -> HttpResponse {
    let name = name.into_inner();
    let action = &body.action;

    match action.as_str() {
        "install" => {
            if data.db.is_module_installed(&name).unwrap_or(false) {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": format!("Module '{}' is already installed", name)
                }));
            }

            let registry = crate::modules::ModuleRegistry::new();
            let module = match registry.get(&name) {
                Some(m) => m,
                None => return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Unknown module: '{}'", name)
                })),
            };

            // Check API keys
            for key in module.required_api_keys() {
                if data.db.get_api_key(key).ok().flatten().is_none() {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": format!("Missing required API key: {}", key)
                    }));
                }
            }

            // Create tables
            if module.has_db_tables() {
                let conn = data.db.conn();
                if let Err(e) = module.init_tables(&conn) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to create tables: {}", e)
                    }));
                }
            }

            let required_keys = module.required_api_keys();
            let key_strs: Vec<&str> = required_keys.iter().copied().collect();
            match data.db.install_module(
                &name,
                module.description(),
                module.version(),
                module.has_db_tables(),
                module.has_tools(),
                module.has_worker(),
                &key_strs,
            ) {
                Ok(_) => {
                    // Install skill if provided
                    if let Some(skill_md) = module.skill_content() {
                        let _ = data.skill_registry.create_skill_from_markdown(skill_md);
                    }

                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "installed",
                        "message": format!("Module '{}' installed. Restart to activate tools and worker.", name)
                    }))
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Install failed: {}", e)
                })),
            }
        }

        "uninstall" => {
            match data.db.uninstall_module(&name) {
                Ok(true) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "uninstalled",
                    "message": format!("Module '{}' uninstalled. Data preserved.", name)
                })),
                Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Module '{}' is not installed", name)
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Uninstall failed: {}", e)
                })),
            }
        }

        "enable" => {
            match data.db.set_module_enabled(&name, true) {
                Ok(true) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "enabled",
                    "message": format!("Module '{}' enabled", name)
                })),
                Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Module '{}' is not installed", name)
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Enable failed: {}", e)
                })),
            }
        }

        "disable" => {
            match data.db.set_module_enabled(&name, false) {
                Ok(true) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "disabled",
                    "message": format!("Module '{}' disabled", name)
                })),
                Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Module '{}' is not installed", name)
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Disable failed: {}", e)
                })),
            }
        }

        _ => HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Unknown action: '{}'. Use 'install', 'uninstall', 'enable', or 'disable'.", action)
        })),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/modules")
            .route("", web::get().to(list_modules))
            .route("/{name}", web::post().to(module_action)),
    );
}
