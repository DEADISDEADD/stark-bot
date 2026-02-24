//! Internal hooks controller — lets modules fire custom persona hooks.
//!
//! `POST /api/internal/hooks/fire` — fires a custom hook event.
//! Authenticated via the `X-Internal-Token` header (same token modules use
//! for other internal-only endpoints).

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::persona_hooks;
use crate::AppState;

#[derive(Deserialize)]
struct FireHookRequest {
    event: String,
    #[serde(default)]
    data: serde_json::Value,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/internal/hooks")
            .route("/fire", web::post().to(fire_hook)),
    );
}

async fn fire_hook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<FireHookRequest>,
) -> HttpResponse {
    // Authenticate via internal token
    let token = req
        .headers()
        .get("X-Internal-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if token.is_empty() || token != state.internal_token {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid or missing X-Internal-Token"
        }));
    }

    let event = body.event.trim();
    if event.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "event is required"
        }));
    }

    log::info!("[HOOKS_API] Firing custom hook event='{}' data={}", event, body.data);

    persona_hooks::fire_custom_hooks(event, body.data.clone(), &state.dispatcher).await;

    HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "event": event,
    }))
}
