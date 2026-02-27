use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::skills::{DbSkillScript, Skill};
use crate::AppState;

#[derive(Serialize)]
pub struct SkillsListResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<SkillInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub enabled: bool,
    pub requires_tools: Vec<String>,
    pub requires_binaries: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

impl From<&Skill> for SkillInfo {
    fn from(skill: &Skill) -> Self {
        SkillInfo {
            name: skill.metadata.name.clone(),
            description: skill.metadata.description.clone(),
            version: skill.metadata.version.clone(),
            source: skill.source.as_str().to_string(),
            enabled: skill.enabled,
            requires_tools: skill.metadata.requires_tools.clone(),
            requires_binaries: skill.metadata.requires_binaries.clone(),
            tags: skill.metadata.tags.clone(),
            homepage: skill.metadata.homepage.clone(),
            metadata: skill.metadata.metadata.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct SkillDetailResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<SkillDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SkillDetail {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub path: String,
    pub enabled: bool,
    pub requires_tools: Vec<String>,
    pub requires_binaries: Vec<String>,
    pub missing_binaries: Vec<String>,
    pub tags: Vec<String>,
    pub arguments: Vec<ArgumentInfo>,
    pub prompt_template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<ScriptInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Serialize)]
pub struct ArgumentInfo {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Serialize)]
pub struct ScriptInfo {
    pub name: String,
    pub language: String,
    /// Script source code (included in detail views)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl From<&DbSkillScript> for ScriptInfo {
    fn from(script: &DbSkillScript) -> Self {
        ScriptInfo {
            name: script.name.clone(),
            language: script.language.clone(),
            code: Some(script.code.clone()),
        }
    }
}

impl From<&Skill> for SkillDetail {
    fn from(skill: &Skill) -> Self {
        let missing_binaries = skill.check_binaries().err().unwrap_or_default();

        let arguments: Vec<ArgumentInfo> = skill
            .metadata
            .arguments
            .iter()
            .map(|(name, arg)| ArgumentInfo {
                name: name.clone(),
                description: arg.description.clone(),
                required: arg.required,
                default: arg.default.clone(),
            })
            .collect();

        SkillDetail {
            name: skill.metadata.name.clone(),
            description: skill.metadata.description.clone(),
            version: skill.metadata.version.clone(),
            source: skill.source.as_str().to_string(),
            path: skill.path.clone(),
            enabled: skill.enabled,
            requires_tools: skill.metadata.requires_tools.clone(),
            requires_binaries: skill.metadata.requires_binaries.clone(),
            missing_binaries,
            tags: skill.metadata.tags.clone(),
            arguments,
            prompt_template: skill.prompt_template.clone(),
            scripts: None,
            homepage: skill.metadata.homepage.clone(),
            metadata: skill.metadata.metadata.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct SetEnabledRequest {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateSkillRequest {
    pub body: String,
}

#[derive(Serialize)]
pub struct OperationResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<SkillInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ScriptsListResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Vec<ScriptInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// --- Skill Graph types ---

#[derive(Serialize)]
pub struct SkillGraphNode {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct SkillGraphEdge {
    pub source: i64,
    pub target: i64,
    pub association_type: String,
    pub strength: f64,
}

#[derive(Serialize)]
pub struct SkillGraphResponse {
    pub success: bool,
    pub nodes: Vec<SkillGraphNode>,
    pub edges: Vec<SkillGraphEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SkillSearchResult {
    pub skill_id: i64,
    pub name: String,
    pub description: String,
    pub similarity: f32,
}

#[derive(Serialize)]
pub struct SkillSearchResponse {
    pub success: bool,
    pub results: Vec<SkillSearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SkillEmbeddingStatsResponse {
    pub success: bool,
    pub total_skills: i64,
    pub skills_with_embeddings: i64,
    pub coverage_percent: f64,
}

#[derive(Deserialize)]
pub struct SkillSearchQuery {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateAssociationRequest {
    pub source_skill_id: i64,
    pub target_skill_id: i64,
    pub association_type: Option<String>,
    pub strength: Option<f64>,
}

// --- StarkHub integration ---

#[derive(Deserialize)]
struct InstallFromHubRequest {
    username: String,
    slug: String,
}

/// GET /api/skills/featured_remote — get featured skills from StarkHub
async fn featured_remote(
    state: web::Data<AppState>,
    _req: HttpRequest,
) -> impl Responder {
    let client = crate::integrations::starkhub_client::StarkHubClient::new();
    let featured = match client.get_featured_skills().await {
        Ok(f) => f,
        Err(e) => {
            log::error!("[SKILLS] Failed to fetch featured skills from StarkHub: {}", e);
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": format!("Failed to fetch from StarkHub: {}", e)
            }));
        }
    };

    // Filter out already-installed skills
    let installed: std::collections::HashSet<String> = state
        .skill_registry
        .list()
        .iter()
        .map(|s| s.metadata.name.clone())
        .collect();

    let filtered: Vec<_> = featured
        .into_iter()
        .filter(|s| {
            let slug_underscore = s.slug.replace('-', "_");
            !installed.contains(&s.slug)
                && !installed.contains(&slug_underscore)
                && !installed.contains(&s.name)
        })
        .collect();

    HttpResponse::Ok().json(filtered)
}

/// POST /api/skills/install_from_hub — install a skill from StarkHub (with file downloads)
async fn install_from_hub(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<InstallFromHubRequest>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let auth_token = req
        .headers()
        .get("X-StarkHub-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    let client = crate::integrations::starkhub_client::StarkHubClient::new();

    // Try ZIP bundle download first (faster — single request for all files)
    if let Ok(Some(zip_bytes)) = client
        .download_bundle("skills", &body.username, &body.slug, &auth_token)
        .await
    {
        match crate::skills::zip_parser::parse_skill_zip(&zip_bytes) {
            Ok(parsed) => {
                let already_exists = state.skill_registry.has_skill(&parsed.name);
                let skill_name = parsed.name.clone();

                // Reconstruct raw markdown from ZIP for registry
                let raw_markdown = reconstruct_skill_md(&parsed);

                match state.skill_registry.create_skill_from_markdown(&raw_markdown) {
                    Ok(_) => {
                        // Write auxiliary files (scripts, ABIs, presets) to disk
                        let mut downloaded_files = vec!["SKILL.md".to_string()];
                        let skills_dir = std::path::PathBuf::from(crate::config::runtime_skills_dir());
                        let skill_folder = skills_dir.join(&skill_name);
                        let _ = std::fs::create_dir_all(&skill_folder);

                        // Write scripts
                        if !parsed.scripts.is_empty() {
                            let scripts_dir = skill_folder.join("scripts");
                            let _ = std::fs::create_dir_all(&scripts_dir);
                            for script in &parsed.scripts {
                                let path = scripts_dir.join(&script.name);
                                if std::fs::write(&path, &script.code).is_ok() {
                                    downloaded_files.push(format!("scripts/{}", script.name));
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        let _ = std::fs::set_permissions(
                                            &path,
                                            std::fs::Permissions::from_mode(0o755),
                                        );
                                    }
                                }
                            }
                        }

                        // Write ABIs
                        if !parsed.abis.is_empty() {
                            let abis_dir = skill_folder.join("abis");
                            let _ = std::fs::create_dir_all(&abis_dir);
                            for abi in &parsed.abis {
                                let filename = format!("{}.json", abi.name);
                                if std::fs::write(abis_dir.join(&filename), &abi.content).is_ok() {
                                    downloaded_files.push(format!("abis/{}", filename));
                                }
                            }
                        }

                        // Write presets
                        if let Some(ref presets) = parsed.presets_content {
                            if std::fs::write(skill_folder.join("web3_presets.ron"), presets).is_ok() {
                                downloaded_files.push("web3_presets.ron".to_string());
                            }
                        }

                        return HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "skill_name": skill_name,
                            "already_existed": already_exists,
                            "files": downloaded_files,
                            "message": format!("{} skill '{}' from @{}/{}",
                                if already_exists { "Updated" } else { "Installed" },
                                skill_name, body.username, body.slug),
                        }));
                    }
                    Err(e) => {
                        log::warn!("[SKILLS] ZIP bundle install failed, falling back: {}", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("[SKILLS] ZIP bundle parse failed, falling back: {}", e);
            }
        }
    }

    // Fallback: individual file downloads (legacy items without bundles)
    let skill_detail = match client.get_skill(&body.username, &body.slug).await {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": format!("Failed to fetch skill from StarkHub: {}", e)
            }));
        }
    };

    let raw_markdown = match skill_detail.get("raw_markdown").and_then(|v| v.as_str()) {
        Some(md) => md.to_string(),
        None => {
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Skill response missing raw_markdown field"
            }));
        }
    };

    let already_exists = match crate::skills::zip_parser::parse_skill_md(&raw_markdown) {
        Ok((meta, _)) => state.skill_registry.has_skill(&meta.name),
        Err(_) => false,
    };

    let db_skill = match state.skill_registry.create_skill_from_markdown(&raw_markdown) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to install skill: {}", e)
            }));
        }
    };

    let skill_name = db_skill.name.clone();

    let mut downloaded_files = Vec::new();
    if let Ok(files) = client
        .list_skill_files(&body.username, &body.slug)
        .await
    {
        if !files.is_empty() {
            let skills_dir = std::path::PathBuf::from(crate::config::runtime_skills_dir());
            let skill_folder = skills_dir.join(&skill_name);
            let _ = std::fs::create_dir_all(&skill_folder);

            for file_summary in &files {
                if let Ok(file_detail) = client
                    .get_skill_file(&body.username, &body.slug, &file_summary.file_name)
                    .await
                {
                    let file_path = skill_folder.join(&file_detail.file_name);
                    if let Err(e) = std::fs::write(&file_path, &file_detail.content) {
                        log::warn!("[SKILLS] Failed to write file '{}': {}", file_detail.file_name, e);
                    } else {
                        downloaded_files.push(file_detail.file_name);
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "skill_name": skill_name,
        "already_existed": already_exists,
        "files": downloaded_files,
        "message": format!("{} skill '{}' from @{}/{}",
            if already_exists { "Updated" } else { "Installed" },
            skill_name, body.username, body.slug),
    }))
}

/// POST /api/skills/publish/{name} — publish a skill to StarkHub (with file uploads)
async fn publish_to_hub(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let auth_token = match req
        .headers()
        .get("X-StarkHub-Token")
        .and_then(|h| h.to_str().ok())
    {
        Some(t) => t.to_string(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "X-StarkHub-Token header required for publishing"
            }));
        }
    };

    let name = path.into_inner();
    let skills_dir = std::path::PathBuf::from(crate::config::runtime_skills_dir());
    let skill_folder = skills_dir.join(&name);

    if !skill_folder.is_dir() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Skill '{}' not found on disk", name)
        }));
    }

    // Find the main .md file (SKILL.md or {name}.md)
    let skill_md_path = skill_folder.join("SKILL.md");
    let named_md_path = skill_folder.join(format!("{}.md", &name));
    let md_path = if skill_md_path.exists() {
        skill_md_path.clone()
    } else if named_md_path.exists() {
        named_md_path.clone()
    } else {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("No SKILL.md or {}.md found in skill folder", name)
        }));
    };

    let raw_markdown = match std::fs::read_to_string(&md_path) {
        Ok(content) => content,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to read skill markdown: {}", e)
            }));
        }
    };

    let client = crate::integrations::starkhub_client::StarkHubClient::new();

    // Publish skill markdown
    let result = match client.publish_skill(&raw_markdown, &auth_token).await {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": format!("Failed to publish to StarkHub: {}", e)
            }));
        }
    };

    let username = result["username"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let slug = result["slug"]
        .as_str()
        .unwrap_or(&name)
        .to_string();

    // Determine the main md filename to skip during file upload
    let main_md_name = md_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    // Upload additional files (everything except the main .md file)
    let mut uploaded_files = Vec::new();
    let mut skipped_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&skill_folder) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name == main_md_name || !entry.path().is_file() {
                continue;
            }
            match std::fs::read_to_string(entry.path()) {
                Ok(content) => {
                    match client
                        .upload_skill_file(&username, &slug, &file_name, &content, &auth_token)
                        .await
                    {
                        Ok(_) => uploaded_files.push(file_name),
                        Err(e) => {
                            log::warn!("[SKILLS] Failed to upload file '{}': {}", file_name, e);
                            skipped_files.push(file_name);
                        }
                    }
                }
                Err(_) => {
                    log::warn!("[SKILLS] Skipping binary file '{}' (not UTF-8)", file_name);
                    skipped_files.push(file_name);
                }
            }
        }
    }

    let mut resp = serde_json::json!({
        "success": true,
        "slug": slug,
        "username": username,
        "uploaded_files": uploaded_files,
        "message": result.get("message").and_then(|m| m.as_str()).unwrap_or("Published"),
    });
    if !skipped_files.is_empty() {
        resp["skipped_files"] = serde_json::json!(skipped_files);
    }
    HttpResponse::Ok().json(resp)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/skills")
            .route("", web::get().to(list_skills))
            .route("/upload", web::post().to(upload_skill))
            .route("/reload", web::post().to(reload_skills))
            .route("/graph", web::get().to(get_skill_graph))
            .route("/graph/search", web::get().to(search_skills_by_embedding))
            .route("/embeddings/stats", web::get().to(get_skill_embedding_stats))
            .route("/embeddings/backfill", web::post().to(backfill_skill_embeddings))
            .route("/associations", web::post().to(create_skill_association))
            .route("/associations/rebuild", web::post().to(rebuild_skill_associations))
            .route("/bundled/available", web::get().to(list_bundled_available))
            .route("/bundled/restore/{name}", web::post().to(restore_bundled_skill))
            .route("/featured_remote", web::get().to(featured_remote))
            .route("/install_from_hub", web::post().to(install_from_hub))
            .route("/publish/{name}", web::post().to(publish_to_hub))
            .route("/{name}", web::get().to(get_skill))
            .route("/{name}", web::put().to(update_skill))
            .route("/{name}", web::delete().to(delete_skill))
            .route("/{name}/enabled", web::put().to(set_enabled))
            .route("/{name}/scripts", web::get().to(get_skill_scripts)),
    );
}

fn validate_session_from_request(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Result<(), HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let token = match token {
        Some(t) => t,
        None => {
            return Err(HttpResponse::Unauthorized().json(OperationResponse {
                success: false,
                message: None,
                error: Some("No authorization token provided".to_string()),
                count: None,
            }));
        }
    };

    match state.db.validate_session(&token) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(HttpResponse::Unauthorized().json(OperationResponse {
            success: false,
            message: None,
            error: Some("Invalid or expired session".to_string()),
            count: None,
        })),
        Err(e) => {
            log::error!("Failed to validate session: {}", e);
            Err(HttpResponse::InternalServerError().json(OperationResponse {
                success: false,
                message: None,
                error: Some("Internal server error".to_string()),
                count: None,
            }))
        }
    }
}

async fn list_skills(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let skills: Vec<SkillInfo> = state
        .skill_registry
        .list()
        .iter()
        .map(|s| s.into())
        .collect();

    HttpResponse::Ok().json(skills)
}

async fn get_skill(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    match state.skill_registry.get(&name) {
        Some(skill) => {
            let mut detail: SkillDetail = (&skill).into();

            // Get associated scripts
            let scripts = state.skill_registry.get_skill_scripts(&name);
            if !scripts.is_empty() {
                detail.scripts = Some(scripts.iter().map(|s| s.into()).collect());
            }

            HttpResponse::Ok().json(SkillDetailResponse {
                success: true,
                skill: Some(detail),
                error: None,
            })
        }
        None => HttpResponse::NotFound().json(SkillDetailResponse {
            success: false,
            skill: None,
            error: Some(format!("Skill '{}' not found", name)),
        }),
    }
}

async fn set_enabled(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<SetEnabledRequest>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    if !state.skill_registry.has_skill(&name) {
        return HttpResponse::NotFound().json(OperationResponse {
            success: false,
            message: None,
            error: Some(format!("Skill '{}' not found", name)),
            count: None,
        });
    }

    // Update in registry (which updates the database)
    state.skill_registry.set_enabled(&name, body.enabled);

    let status = if body.enabled { "enabled" } else { "disabled" };
    HttpResponse::Ok().json(OperationResponse {
        success: true,
        message: Some(format!("Skill '{}' {}", name, status)),
        error: None,
        count: None,
    })
}

async fn reload_skills(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    match state.skill_registry.reload().await {
        Ok(count) => {
            // Background-backfill embeddings for any newly loaded skills
            if let Some(ref engine) = state.hybrid_search {
                let emb_gen = engine.embedding_generator().clone();
                let db = state.db.clone();
                tokio::spawn(async move {
                    if let Err(e) = crate::skills::embeddings::backfill_skill_embeddings(&db, &emb_gen).await {
                        log::warn!("[SKILL-EMB] Post-reload backfill failed: {}", e);
                    }
                });
            }

            HttpResponse::Ok().json(OperationResponse {
                success: true,
                message: Some(format!("Loaded {} skills from disk", count)),
                error: None,
                count: Some(state.skill_registry.len()),
            })
        }
        Err(e) => {
            log::error!("Failed to reload skills: {}", e);
            HttpResponse::InternalServerError().json(OperationResponse {
                success: false,
                message: None,
                error: Some(format!("Failed to reload skills: {}", e)),
                count: None,
            })
        }
    }
}

async fn upload_skill(
    state: web::Data<AppState>,
    req: HttpRequest,
    mut payload: Multipart,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    // Read the uploaded file and capture filename
    let mut file_data: Vec<u8> = Vec::new();
    let mut filename: Option<String> = None;

    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                // Capture filename from content disposition
                if filename.is_none() {
                    filename = field.content_disposition()
                        .get_filename()
                        .map(|s| s.to_string());
                }

                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(data) => file_data.extend_from_slice(&data),
                        Err(e) => {
                            return HttpResponse::BadRequest().json(UploadResponse {
                                success: false,
                                skill: None,
                                error: Some(format!("Failed to read upload data: {}", e)),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                return HttpResponse::BadRequest().json(UploadResponse {
                    success: false,
                    skill: None,
                    error: Some(format!("Failed to process upload: {}", e)),
                });
            }
        }
    }

    if file_data.is_empty() {
        return HttpResponse::BadRequest().json(UploadResponse {
            success: false,
            skill: None,
            error: Some("No file uploaded".to_string()),
        });
    }

    // Reject uploads larger than 10MB (ZIP bomb protection)
    if file_data.len() > crate::disk_quota::MAX_SKILL_ZIP_BYTES {
        return HttpResponse::BadRequest().json(UploadResponse {
            success: false,
            skill: None,
            error: Some(format!(
                "Upload rejected: file size ({} bytes) exceeds the 10MB limit for skill uploads.",
                file_data.len()
            )),
        });
    }

    // Determine file type from filename or content
    let is_markdown = filename
        .as_ref()
        .map(|f| f.to_lowercase().ends_with(".md"))
        .unwrap_or(false);

    // Parse and create the skill based on file type
    let result = if is_markdown {
        // Parse as markdown file
        match String::from_utf8(file_data) {
            Ok(content) => state.skill_registry.create_skill_from_markdown(&content),
            Err(e) => Err(format!("Invalid UTF-8 in markdown file: {}", e)),
        }
    } else {
        // Parse as ZIP file
        state.skill_registry.create_skill_from_zip(&file_data)
    };

    match result {
        Ok(db_skill) => {
            // Load the new skill's ABIs and presets into memory
            if let Some(skill_id) = db_skill.id {
                // Load ABIs for this skill into the in-memory index
                if let Ok(abis) = state.db.get_skill_abis(skill_id) {
                    for abi in abis {
                        crate::web3::register_abi_content(&abi.name, &abi.content);
                    }
                }
                // Load presets for this skill into the in-memory index
                crate::tools::presets::load_skill_presets_from_db(&state.db, skill_id);
            }

            // Auto-generate embedding + rebuild associations for the new skill
            if let Some(skill_id) = db_skill.id {
                if let Some(ref engine) = state.hybrid_search {
                    let emb_gen = engine.embedding_generator().clone();
                    let db = state.db.clone();
                    let skill_name = db_skill.name.clone();
                    let emb_text = crate::skills::embeddings::build_skill_embedding_text(&db_skill);
                    tokio::spawn(async move {
                        if let Ok(embedding) = emb_gen.generate(&emb_text).await {
                            let dims = embedding.len() as i32;
                            if let Err(e) = db.upsert_skill_embedding(skill_id, &embedding, "remote", dims) {
                                log::warn!("[SKILL-EMB] Failed to auto-embed skill '{}': {}", skill_name, e);
                            } else {
                                log::info!("[SKILL-EMB] Auto-embedded skill '{}'", skill_name);
                                // Rebuild associations for this skill
                                if let Err(e) = crate::skills::embeddings::rebuild_associations_for_skill(&db, skill_id, 0.30).await {
                                    log::warn!("[SKILL-ASSOC] Failed to rebuild associations for '{}': {}", skill_name, e);
                                }
                            }
                        }
                    });
                }
            }

            let skill = db_skill.into_skill();
            HttpResponse::Ok().json(UploadResponse {
                success: true,
                skill: Some((&skill).into()),
                error: None,
            })
        }
        Err(e) => {
            log::error!("Failed to create skill: {}", e);
            HttpResponse::BadRequest().json(UploadResponse {
                success: false,
                skill: None,
                error: Some(e),
            })
        }
    }
}

async fn update_skill(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateSkillRequest>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    // Get the existing skill
    let existing = match state.skill_registry.get(&name) {
        Some(skill) => skill,
        None => {
            return HttpResponse::NotFound().json(SkillDetailResponse {
                success: false,
                skill: None,
                error: Some(format!("Skill '{}' not found", name)),
            });
        }
    };

    // Build an updated DbSkill with the new body
    let now = chrono::Utc::now().to_rfc3339();
    let db_skill = crate::skills::DbSkill {
        id: None,
        name: existing.metadata.name.clone(),
        description: existing.metadata.description.clone(),
        body: body.body.clone(),
        version: existing.metadata.version.clone(),
        author: existing.metadata.author.clone(),
        homepage: existing.metadata.homepage.clone(),
        metadata: existing.metadata.metadata.clone(),
        enabled: existing.enabled,
        requires_tools: existing.metadata.requires_tools.clone(),
        requires_binaries: existing.metadata.requires_binaries.clone(),
        arguments: existing.metadata.arguments.clone(),
        tags: existing.metadata.tags.clone(),
        subagent_type: existing.metadata.subagent_type.clone(),
        requires_api_keys: existing.metadata.requires_api_keys.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    // Write updated SKILL.md to disk
    let skills_dir = std::path::PathBuf::from(crate::config::runtime_skills_dir());
    let skill_folder = skills_dir.join(&name);
    if skill_folder.exists() {
        // Find the existing .md file (SKILL.md or {name}.md)
        let skill_md_path = skill_folder.join("SKILL.md");
        let named_md_path = skill_folder.join(format!("{}.md", &name));
        let md_path = if skill_md_path.exists() {
            skill_md_path
        } else if named_md_path.exists() {
            named_md_path
        } else {
            skill_folder.join("SKILL.md")
        };

        // Read existing file, replace body, write back
        if let Ok(content) = std::fs::read_to_string(&md_path) {
            let trimmed = content.trim();
            if trimmed.starts_with("---") {
                let rest = &trimmed[3..];
                if let Some(end_idx) = rest.find("---") {
                    let frontmatter = &rest[..end_idx + 3]; // include closing ---
                    let updated = format!("---{}\n\n{}", frontmatter, body.body);
                    if let Err(e) = std::fs::write(&md_path, &updated) {
                        log::warn!("Failed to write updated SKILL.md for '{}': {}", name, e);
                    }
                }
            }
        } else {
            // No existing file — reconstruct from DbSkill
            let md_content = crate::skills::reconstruct_skill_md_from_db(&db_skill);
            if let Err(e) = std::fs::write(&md_path, &md_content) {
                log::warn!("Failed to write SKILL.md for '{}': {}", name, e);
            }
        }
    } else {
        // Skill folder doesn't exist — create it
        let md_content = crate::skills::reconstruct_skill_md_from_db(&db_skill);
        let parsed = crate::skills::ParsedSkill {
            name: db_skill.name.clone(),
            description: db_skill.description.clone(),
            body: db_skill.body.clone(),
            version: db_skill.version.clone(),
            author: db_skill.author.clone(),
            homepage: db_skill.homepage.clone(),
            metadata: db_skill.metadata.clone(),
            requires_tools: db_skill.requires_tools.clone(),
            requires_binaries: db_skill.requires_binaries.clone(),
            arguments: db_skill.arguments.clone(),
            tags: db_skill.tags.clone(),
            subagent_type: db_skill.subagent_type.clone(),
            requires_api_keys: db_skill.requires_api_keys.clone(),
            scripts: Vec::new(),
            abis: Vec::new(),
            presets_content: None,
            flows: Vec::new(),
        };
        if let Err(e) = crate::skills::write_skill_folder(&skills_dir, &parsed) {
            log::warn!("Failed to create skill folder for '{}': {}", name, e);
        }
    }

    // Force-update in database (bypass version check)
    if let Err(e) = state.db.create_skill_force(&db_skill) {
        log::error!("Failed to update skill '{}': {}", name, e);
        return HttpResponse::InternalServerError().json(SkillDetailResponse {
            success: false,
            skill: None,
            error: Some(format!("Failed to update skill: {}", e)),
        });
    }

    // Auto-regenerate embedding + rebuild associations for the updated skill
    if let Some(ref engine) = state.hybrid_search {
        if let Ok(Some(updated)) = state.db.get_skill(&name) {
            if let Some(skill_id) = updated.id {
                let emb_gen = engine.embedding_generator().clone();
                let db = state.db.clone();
                let skill_name = name.clone();
                tokio::spawn(async move {
                    let text = crate::skills::embeddings::build_skill_embedding_text(&updated);
                    if let Ok(embedding) = emb_gen.generate(&text).await {
                        let dims = embedding.len() as i32;
                        if let Err(e) = db.upsert_skill_embedding(skill_id, &embedding, "remote", dims) {
                            log::warn!("[SKILL-EMB] Failed to re-embed skill '{}': {}", skill_name, e);
                        } else {
                            // Rebuild associations for this skill
                            if let Err(e) = crate::skills::embeddings::rebuild_associations_for_skill(&db, skill_id, 0.30).await {
                                log::warn!("[SKILL-ASSOC] Failed to rebuild associations for '{}': {}", skill_name, e);
                            }
                        }
                    }
                });
            }
        }
    }

    // Re-fetch the updated skill
    match state.skill_registry.get(&name) {
        Some(skill) => {
            let mut detail: SkillDetail = (&skill).into();
            let scripts = state.skill_registry.get_skill_scripts(&name);
            if !scripts.is_empty() {
                detail.scripts = Some(scripts.iter().map(|s| s.into()).collect());
            }
            HttpResponse::Ok().json(SkillDetailResponse {
                success: true,
                skill: Some(detail),
                error: None,
            })
        }
        None => HttpResponse::InternalServerError().json(SkillDetailResponse {
            success: false,
            skill: None,
            error: Some("Skill not found after update".to_string()),
        }),
    }
}

async fn delete_skill(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    // Get skill ID before deleting (for association cleanup)
    let skill_id = state.db.get_skill(&name).ok().flatten().and_then(|s| s.id);

    if skill_id.is_none() && !state.skill_registry.has_skill(&name) {
        return HttpResponse::NotFound().json(OperationResponse {
            success: false,
            message: None,
            error: Some(format!("Skill '{}' not found", name)),
            count: None,
        });
    }

    match state.skill_registry.delete_skill(&name) {
        Ok(true) => {
            // Clean up associations for the deleted skill
            if let Some(sid) = skill_id {
                if let Err(e) = state.db.delete_skill_associations_for(sid) {
                    log::warn!("[SKILL-ASSOC] Failed to clean up associations for deleted skill '{}': {}", name, e);
                }
            }
            HttpResponse::Ok().json(OperationResponse {
                success: true,
                message: Some(format!("Skill '{}' deleted", name)),
                error: None,
                count: None,
            })
        }
        Ok(false) => HttpResponse::NotFound().json(OperationResponse {
            success: false,
            message: None,
            error: Some(format!("Skill '{}' not found", name)),
            count: None,
        }),
        Err(e) => {
            log::error!("Failed to delete skill: {}", e);
            HttpResponse::InternalServerError().json(OperationResponse {
                success: false,
                message: None,
                error: Some(format!("Failed to delete skill: {}", e)),
                count: None,
            })
        }
    }
}

async fn list_bundled_available(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let available = state.skill_registry.list_bundled_available().await;
    HttpResponse::Ok().json(available)
}

async fn restore_bundled_skill(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    match state.skill_registry.restore_bundled_skill(&name).await {
        Ok(db_skill) => {
            // Load ABIs and presets into memory (same pattern as upload_skill)
            if let Some(skill_id) = db_skill.id {
                if let Ok(abis) = state.db.get_skill_abis(skill_id) {
                    for abi in abis {
                        crate::web3::register_abi_content(&abi.name, &abi.content);
                    }
                }
                crate::tools::presets::load_skill_presets_from_db(&state.db, skill_id);
            }

            // Auto-generate embedding + rebuild associations
            if let Some(skill_id) = db_skill.id {
                if let Some(ref engine) = state.hybrid_search {
                    let emb_gen = engine.embedding_generator().clone();
                    let db = state.db.clone();
                    let skill_name = db_skill.name.clone();
                    let emb_text = crate::skills::embeddings::build_skill_embedding_text(&db_skill);
                    tokio::spawn(async move {
                        if let Ok(embedding) = emb_gen.generate(&emb_text).await {
                            let dims = embedding.len() as i32;
                            if let Err(e) = db.upsert_skill_embedding(skill_id, &embedding, "remote", dims) {
                                log::warn!("[SKILL-EMB] Failed to auto-embed restored skill '{}': {}", skill_name, e);
                            } else {
                                log::info!("[SKILL-EMB] Auto-embedded restored skill '{}'", skill_name);
                                if let Err(e) = crate::skills::embeddings::rebuild_associations_for_skill(&db, skill_id, 0.30).await {
                                    log::warn!("[SKILL-ASSOC] Failed to rebuild associations for '{}': {}", skill_name, e);
                                }
                            }
                        }
                    });
                }
            }

            let skill = db_skill.into_skill();
            HttpResponse::Ok().json(UploadResponse {
                success: true,
                skill: Some((&skill).into()),
                error: None,
            })
        }
        Err(e) => {
            log::error!("Failed to restore bundled skill '{}': {}", name, e);
            HttpResponse::BadRequest().json(UploadResponse {
                success: false,
                skill: None,
                error: Some(e),
            })
        }
    }
}

async fn get_skill_scripts(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let name = path.into_inner();

    if !state.skill_registry.has_skill(&name) {
        return HttpResponse::NotFound().json(ScriptsListResponse {
            success: false,
            scripts: None,
            error: Some(format!("Skill '{}' not found", name)),
        });
    }

    let scripts = state.skill_registry.get_skill_scripts(&name);
    let script_infos: Vec<ScriptInfo> = scripts.iter().map(|s| s.into()).collect();

    HttpResponse::Ok().json(ScriptsListResponse {
        success: true,
        scripts: Some(script_infos),
        error: None,
    })
}

// --- Skill Graph Endpoints ---

async fn get_skill_graph(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let skills = match state.db.list_skills() {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(SkillGraphResponse {
                success: false,
                nodes: vec![],
                edges: vec![],
                error: Some(format!("Failed to list skills: {}", e)),
            });
        }
    };

    let nodes: Vec<SkillGraphNode> = skills
        .iter()
        .filter_map(|s| {
            s.id.map(|id| SkillGraphNode {
                id,
                name: s.name.clone(),
                description: s.description.clone(),
                tags: s.tags.clone(),
                enabled: s.enabled,
            })
        })
        .collect();

    let edges = match state.db.list_all_skill_associations() {
        Ok(assocs) => assocs
            .into_iter()
            .map(|a| SkillGraphEdge {
                source: a.source_skill_id,
                target: a.target_skill_id,
                association_type: a.association_type,
                strength: a.strength,
            })
            .collect(),
        Err(_) => vec![],
    };

    HttpResponse::Ok().json(SkillGraphResponse {
        success: true,
        nodes,
        edges,
        error: None,
    })
}

async fn search_skills_by_embedding(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SkillSearchQuery>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let limit = query.limit.unwrap_or(5);

    // Try embedding search first if engine is available
    if let Some(ref engine) = state.hybrid_search {
        let emb_gen = engine.embedding_generator().clone();
        // Check if any embeddings exist
        let has_embeddings = state.db.count_skill_embeddings().unwrap_or(0) > 0;

        if has_embeddings {
            match crate::skills::embeddings::search_skills(&state.db, &emb_gen, &query.query, limit, 0.20).await {
                Ok(matches) if !matches.is_empty() => {
                    let results: Vec<SkillSearchResult> = matches
                        .into_iter()
                        .map(|(skill, sim)| SkillSearchResult {
                            skill_id: skill.id.unwrap_or(0),
                            name: skill.name,
                            description: skill.description,
                            similarity: sim,
                        })
                        .collect();
                    return HttpResponse::Ok().json(SkillSearchResponse {
                        success: true,
                        results,
                        error: None,
                    });
                }
                Ok(_) => { /* empty results — fall through to text search */ }
                Err(e) => {
                    log::warn!("[SKILL-SEARCH] Embedding search failed, falling back to text: {}", e);
                }
            }
        }
    }

    // Fallback: text-based search (works without embeddings)
    match crate::skills::embeddings::search_skills_text(&state.db, &query.query, limit) {
        Ok(matches) => {
            let results: Vec<SkillSearchResult> = matches
                .into_iter()
                .map(|(skill, sim)| SkillSearchResult {
                    skill_id: skill.id.unwrap_or(0),
                    name: skill.name,
                    description: skill.description,
                    similarity: sim,
                })
                .collect();
            HttpResponse::Ok().json(SkillSearchResponse {
                success: true,
                results,
                error: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(SkillSearchResponse {
            success: false,
            results: vec![],
            error: Some(e),
        }),
    }
}

async fn get_skill_embedding_stats(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let total_skills = state.db.list_enabled_skills()
        .map(|s| s.len() as i64)
        .unwrap_or(0);
    let skills_with_embeddings = state.db.count_skill_embeddings().unwrap_or(0);
    let coverage = if total_skills > 0 {
        (skills_with_embeddings as f64 / total_skills as f64) * 100.0
    } else {
        0.0
    };

    HttpResponse::Ok().json(SkillEmbeddingStatsResponse {
        success: true,
        total_skills,
        skills_with_embeddings,
        coverage_percent: coverage,
    })
}

async fn backfill_skill_embeddings(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let engine = match &state.hybrid_search {
        Some(e) => e,
        None => {
            return HttpResponse::ServiceUnavailable().json(OperationResponse {
                success: false,
                message: None,
                error: Some("Embedding engine not configured".to_string()),
                count: None,
            });
        }
    };

    let emb_gen = engine.embedding_generator().clone();

    match crate::skills::embeddings::backfill_skill_embeddings(&state.db, &emb_gen).await {
        Ok(count) => HttpResponse::Ok().json(OperationResponse {
            success: true,
            message: Some(format!("Generated {} skill embeddings", count)),
            error: None,
            count: Some(count),
        }),
        Err(e) => HttpResponse::InternalServerError().json(OperationResponse {
            success: false,
            message: None,
            error: Some(e),
            count: None,
        }),
    }
}

async fn create_skill_association(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateAssociationRequest>,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let assoc_type = body.association_type.as_deref().unwrap_or("related");
    let strength = body.strength.unwrap_or(0.5);

    match state.db.create_skill_association(
        body.source_skill_id,
        body.target_skill_id,
        assoc_type,
        strength,
        None,
    ) {
        Ok(id) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "id": id,
        })),
        Err(e) => HttpResponse::InternalServerError().json(OperationResponse {
            success: false,
            message: None,
            error: Some(format!("Failed to create association: {}", e)),
            count: None,
        }),
    }
}

async fn rebuild_skill_associations(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session_from_request(&state, &req) {
        return resp;
    }

    let engine = match &state.hybrid_search {
        Some(e) => e,
        None => {
            return HttpResponse::ServiceUnavailable().json(OperationResponse {
                success: false,
                message: None,
                error: Some("Embedding engine not configured".to_string()),
                count: None,
            });
        }
    };

    let emb_gen = engine.embedding_generator().clone();

    match crate::skills::embeddings::rebuild_skill_associations(&state.db, &emb_gen, 0.30).await {
        Ok(count) => HttpResponse::Ok().json(OperationResponse {
            success: true,
            message: Some(format!("Rebuilt {} skill associations", count)),
            error: None,
            count: Some(count),
        }),
        Err(e) => HttpResponse::InternalServerError().json(OperationResponse {
            success: false,
            message: None,
            error: Some(e),
            count: None,
        }),
    }
}

/// Reconstruct SKILL.md from a ParsedSkill so we can pass it to create_skill_from_markdown.
fn reconstruct_skill_md(parsed: &crate::skills::zip_parser::ParsedSkill) -> String {
    let mut fm = format!("name: {}\ndescription: {}\nversion: {}\n", parsed.name, parsed.description, parsed.version);

    if let Some(ref author) = parsed.author {
        fm.push_str(&format!("author: {}\n", author));
    }
    if let Some(ref homepage) = parsed.homepage {
        fm.push_str(&format!("homepage: {}\n", homepage));
    }
    if !parsed.requires_tools.is_empty() {
        fm.push_str(&format!("requires_tools: [{}]\n", parsed.requires_tools.join(", ")));
    }
    if !parsed.requires_binaries.is_empty() {
        fm.push_str(&format!("requires_binaries: [{}]\n", parsed.requires_binaries.join(", ")));
    }
    if !parsed.tags.is_empty() {
        fm.push_str(&format!("tags: [{}]\n", parsed.tags.join(", ")));
    }
    if let Some(ref subagent_type) = parsed.subagent_type {
        fm.push_str(&format!("subagent_type: {}\n", subagent_type));
    }
    if !parsed.arguments.is_empty() {
        fm.push_str("arguments:\n");
        for (key, arg) in &parsed.arguments {
            fm.push_str(&format!("  {}:\n    description: \"{}\"\n", key, arg.description));
            if let Some(ref default) = arg.default {
                fm.push_str(&format!("    default: \"{}\"\n", default));
            }
        }
    }
    if !parsed.requires_api_keys.is_empty() {
        fm.push_str("requires_api_keys:\n");
        for (key, api_key) in &parsed.requires_api_keys {
            fm.push_str(&format!("  {}:\n    description: \"{}\"\n", key, api_key.description));
        }
    }

    format!("---\n{}\n---\n{}", fm.trim(), parsed.body)
}
