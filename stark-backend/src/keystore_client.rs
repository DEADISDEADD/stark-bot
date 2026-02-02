//! Keystore API client with SIWE authentication
//!
//! Handles authenticated access to the keystore.defirelay.com API for
//! storing and retrieving encrypted API key backups.

use ethers::signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

const KEYSTORE_API: &str = "https://keystore.defirelay.com";

/// Cached session for keystore API
#[derive(Debug, Clone)]
struct KeystoreSession {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe keystore client with session caching
pub struct KeystoreClient {
    session: Arc<RwLock<Option<KeystoreSession>>>,
    http_client: reqwest::Client,
}

// Request/Response types for keystore API

#[derive(Serialize)]
struct AuthorizeRequest {
    address: String,
}

#[derive(Deserialize)]
struct AuthorizeResponse {
    success: bool,
    message: Option<String>,
    nonce: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct VerifyRequest {
    address: String,
    signature: String,
}

#[derive(Deserialize)]
struct VerifyResponse {
    success: bool,
    token: Option<String>,
    expires_at: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct StoreKeysRequest {
    encrypted_data: String,
    key_count: usize,
}

#[derive(Deserialize)]
pub struct StoreKeysResponse {
    pub success: bool,
    pub message: Option<String>,
    pub key_count: Option<usize>,
    pub updated_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct GetKeysResponse {
    pub success: bool,
    pub encrypted_data: Option<String>,
    pub key_count: Option<usize>,
    pub updated_at: Option<String>,
    pub error: Option<String>,
}

impl KeystoreClient {
    /// Create a new keystore client
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Check if current session is valid (exists and not expired)
    async fn is_session_valid(&self) -> bool {
        let session = self.session.read().await;
        if let Some(ref s) = *session {
            // Add 60 second buffer before expiry
            s.expires_at > chrono::Utc::now() + chrono::Duration::seconds(60)
        } else {
            false
        }
    }

    /// Get the current session token if valid
    async fn get_token(&self) -> Option<String> {
        let session = self.session.read().await;
        if let Some(ref s) = *session {
            if s.expires_at > chrono::Utc::now() + chrono::Duration::seconds(60) {
                return Some(s.token.clone());
            }
        }
        None
    }

    /// Authenticate with the keystore server using SIWE
    async fn authenticate(&self, private_key: &str) -> Result<String, String> {
        // Parse wallet from private key
        let pk_clean = private_key.trim_start_matches("0x");
        let wallet: LocalWallet = pk_clean
            .parse()
            .map_err(|e| format!("Invalid private key: {:?}", e))?;
        let address = format!("{:?}", wallet.address());

        log::info!("[Keystore] Authenticating wallet: {}", address);

        // Step 1: Request challenge
        let auth_resp = self
            .http_client
            .post(format!("{}/api/authorize", KEYSTORE_API))
            .json(&AuthorizeRequest {
                address: address.clone(),
            })
            .send()
            .await
            .map_err(|e| format!("Failed to connect to keystore: {}", e))?;

        if !auth_resp.status().is_success() {
            return Err(format!(
                "Keystore authorize failed with status: {}",
                auth_resp.status()
            ));
        }

        let auth_data: AuthorizeResponse = auth_resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse authorize response: {}", e))?;

        if !auth_data.success {
            return Err(auth_data.error.unwrap_or_else(|| "Authorization failed".to_string()));
        }

        let message = auth_data
            .message
            .ok_or_else(|| "No challenge message in response".to_string())?;

        log::debug!("[Keystore] Got challenge, signing...");

        // Step 2: Sign the SIWE message
        let signature = wallet
            .sign_message(&message)
            .await
            .map_err(|e| format!("Failed to sign message: {:?}", e))?;
        let signature_hex = format!("0x{}", hex::encode(signature.to_vec()));

        // Step 3: Verify signature and get token
        let verify_resp = self
            .http_client
            .post(format!("{}/api/authorize/verify", KEYSTORE_API))
            .json(&VerifyRequest {
                address: address.clone(),
                signature: signature_hex,
            })
            .send()
            .await
            .map_err(|e| format!("Failed to verify signature: {}", e))?;

        if !verify_resp.status().is_success() {
            return Err(format!(
                "Keystore verify failed with status: {}",
                verify_resp.status()
            ));
        }

        let verify_data: VerifyResponse = verify_resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse verify response: {}", e))?;

        if !verify_data.success {
            return Err(verify_data.error.unwrap_or_else(|| "Verification failed".to_string()));
        }

        let token = verify_data
            .token
            .ok_or_else(|| "No token in response".to_string())?;

        let expires_at = if let Some(exp) = verify_data.expires_at {
            chrono::DateTime::parse_from_rfc3339(&exp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now() + chrono::Duration::hours(1))
        } else {
            chrono::Utc::now() + chrono::Duration::hours(1)
        };

        // Cache the session
        let mut session = self.session.write().await;
        *session = Some(KeystoreSession {
            token: token.clone(),
            expires_at,
        });

        log::info!("[Keystore] Authentication successful, token expires at {}", expires_at);

        Ok(token)
    }

    /// Ensure we have a valid session, authenticating if needed
    async fn ensure_authenticated(&self, private_key: &str) -> Result<String, String> {
        if let Some(token) = self.get_token().await {
            return Ok(token);
        }
        self.authenticate(private_key).await
    }

    /// Store encrypted keys to the keystore
    pub async fn store_keys(
        &self,
        private_key: &str,
        encrypted_data: &str,
        key_count: usize,
    ) -> Result<StoreKeysResponse, String> {
        let token = self.ensure_authenticated(private_key).await?;

        let resp = self
            .http_client
            .post(format!("{}/api/store_keys", KEYSTORE_API))
            .header("Authorization", format!("Bearer {}", token))
            .json(&StoreKeysRequest {
                encrypted_data: encrypted_data.to_string(),
                key_count,
            })
            .send()
            .await
            .map_err(|e| format!("Failed to connect to keystore: {}", e))?;

        // If unauthorized, try re-authenticating once
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            log::warn!("[Keystore] Token expired, re-authenticating...");
            let new_token = self.authenticate(private_key).await?;

            let retry_resp = self
                .http_client
                .post(format!("{}/api/store_keys", KEYSTORE_API))
                .header("Authorization", format!("Bearer {}", new_token))
                .json(&StoreKeysRequest {
                    encrypted_data: encrypted_data.to_string(),
                    key_count,
                })
                .send()
                .await
                .map_err(|e| format!("Failed to connect to keystore: {}", e))?;

            return retry_resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get encrypted keys from the keystore
    pub async fn get_keys(&self, private_key: &str) -> Result<GetKeysResponse, String> {
        let token = self.ensure_authenticated(private_key).await?;

        let resp = self
            .http_client
            .post(format!("{}/api/get_keys", KEYSTORE_API))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to keystore: {}", e))?;

        // If unauthorized, try re-authenticating once
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            log::warn!("[Keystore] Token expired, re-authenticating...");
            let new_token = self.authenticate(private_key).await?;

            let retry_resp = self
                .http_client
                .post(format!("{}/api/get_keys", KEYSTORE_API))
                .header("Authorization", format!("Bearer {}", new_token))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to keystore: {}", e))?;

            return retry_resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e));
        }

        // Handle 404 specifically
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(GetKeysResponse {
                success: false,
                encrypted_data: None,
                key_count: None,
                updated_at: None,
                error: Some("No backup found for this wallet".to_string()),
            });
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Clear the cached session (for testing or logout)
    pub async fn clear_session(&self) {
        let mut session = self.session.write().await;
        *session = None;
    }
}

impl Default for KeystoreClient {
    fn default() -> Self {
        Self::new()
    }
}

// Global singleton for the keystore client
lazy_static::lazy_static! {
    pub static ref KEYSTORE_CLIENT: KeystoreClient = KeystoreClient::new();
}
