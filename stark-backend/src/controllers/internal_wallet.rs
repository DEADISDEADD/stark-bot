use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ── Request / Response types ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    /// "utf8" (default) or "hex"
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf8".to_string()
}

#[derive(Debug, Serialize)]
pub struct SignMessageResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub success: bool,
    pub address: String,
}

// ── Auth helper ─────────────────────────────────────────────────────────

fn validate_internal_token(state: &web::Data<AppState>, req: &HttpRequest) -> Result<(), HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let token = match token {
        Some(t) => t,
        None => {
            return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "No authorization token provided"
            })));
        }
    };

    // Constant-time comparison to prevent timing attacks
    let expected = state.internal_token.as_bytes();
    let provided = token.as_bytes();
    let mut diff = expected.len() ^ provided.len();
    for (a, b) in expected.iter().zip(provided.iter()) {
        diff |= (*a ^ *b) as usize;
    }
    if diff != 0 {
        return Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid internal token"
        })));
    }

    Ok(())
}

// ── Handlers ────────────────────────────────────────────────────────────

async fn get_address(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(resp) = validate_internal_token(&state, &req) {
        return resp;
    }

    match &state.wallet_provider {
        Some(wp) => HttpResponse::Ok().json(AddressResponse {
            success: true,
            address: wp.get_address(),
        }),
        None => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "success": false,
            "error": "No wallet provider configured"
        })),
    }
}

async fn sign_message(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<SignMessageRequest>,
) -> HttpResponse {
    if let Err(resp) = validate_internal_token(&state, &req) {
        return resp;
    }

    let wp = match &state.wallet_provider {
        Some(wp) => wp,
        None => {
            return HttpResponse::ServiceUnavailable().json(SignMessageResponse {
                success: false,
                signature: None,
                address: None,
                error: Some("No wallet provider configured".to_string()),
            });
        }
    };

    // Decode message bytes based on encoding
    let msg_bytes = match body.encoding.as_str() {
        "hex" => {
            let hex_str = body.message.strip_prefix("0x").unwrap_or(&body.message);
            match hex::decode(hex_str) {
                Ok(b) => b,
                Err(e) => {
                    return HttpResponse::BadRequest().json(SignMessageResponse {
                        success: false,
                        signature: None,
                        address: None,
                        error: Some(format!("Invalid hex encoding: {}", e)),
                    });
                }
            }
        }
        _ => body.message.as_bytes().to_vec(),
    };

    match wp.sign_message(&msg_bytes).await {
        Ok(sig) => {
            let sig_hex = format!("0x{}", sig);
            HttpResponse::Ok().json(SignMessageResponse {
                success: true,
                signature: Some(sig_hex),
                address: Some(wp.get_address()),
                error: None,
            })
        }
        Err(e) => {
            log::error!("[INTERNAL_WALLET] sign_message failed: {}", e);
            HttpResponse::InternalServerError().json(SignMessageResponse {
                success: false,
                signature: None,
                address: None,
                error: Some(format!("Signing failed: {}", e)),
            })
        }
    }
}

// ── Route config ────────────────────────────────────────────────────────

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/internal/wallet")
            .route("/address", web::get().to(get_address))
            .route("/sign-message", web::post().to(sign_message)),
    );
}
