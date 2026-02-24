//! StarkHub API client — search, download, and publish modules from hub.starkbot.ai

use serde::{Deserialize, Serialize};

const DEFAULT_HUB_URL: &str = "https://hub.starkbot.ai/api";

/// Client for the StarkHub module registry API.
pub struct StarkHubClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author_address: String,
    pub author_username: Option<String>,
    pub tools_provided: Vec<String>,
    pub install_count: i32,
    pub featured: Option<bool>,
    pub license: Option<String>,
    pub x402_cost: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDetail {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: ModuleAuthor,
    pub manifest: serde_json::Value,
    pub tools_provided: Vec<String>,
    pub install_count: i32,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub platforms: Vec<PlatformBinary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthor {
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformBinary {
    pub platform: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFileSummary {
    pub file_name: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub download_url: String,
    pub sha256_checksum: String,
    pub file_size: i64,
    pub version: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

// --- Agent Subtype types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubtypeSummary {
    pub id: String,
    pub slug: String,
    pub key: String,
    pub label: String,
    pub emoji: String,
    pub description: String,
    pub version: String,
    pub author_username: Option<String>,
    pub author_address: String,
    pub install_count: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubtypeDetail {
    pub id: String,
    pub slug: String,
    pub key: String,
    pub label: String,
    pub emoji: String,
    pub description: String,
    pub version: String,
    pub author: AgentSubtypeAuthor,
    pub raw_agent_md: String,
    pub prompt: String,
    pub metadata: serde_json::Value,
    pub install_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubtypeAuthor {
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubtypeFileSummary {
    pub file_name: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSubtypeFileDetail {
    pub file_name: String,
    pub content: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

// --- Skill types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author_username: Option<String>,
    pub author_address: String,
    pub install_count: i32,
    pub featured: Option<bool>,
    pub tags: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileSummary {
    pub file_name: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileDetail {
    pub file_name: String,
    pub content: String,
    pub file_size: i64,
    pub sha256_checksum: String,
}

impl StarkHubClient {
    pub fn new() -> Self {
        let base_url = std::env::var("STARKHUB_API_URL")
            .unwrap_or_else(|_| DEFAULT_HUB_URL.to_string());
        Self {
            base_url,
            http: reqwest::Client::new(),
        }
    }

    /// Get featured modules from StarkHub.
    pub async fn get_featured_modules(&self) -> Result<Vec<ModuleSummary>, String> {
        let url = format!("{}/modules/featured", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Search modules on StarkHub.
    pub async fn search_modules(&self, query: &str) -> Result<Vec<ModuleSummary>, String> {
        let url = format!("{}/modules/search", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        let paginated: PaginatedResponse<ModuleSummary> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(paginated.data)
    }

    /// Get module detail by @username/slug.
    pub async fn get_module(&self, username: &str, slug: &str) -> Result<ModuleDetail, String> {
        let url = format!("{}/modules/@{}/{}", self.base_url, username, slug);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if resp.status().as_u16() == 404 {
            return Err(format!("Module @{}/{} not found on StarkHub", username, slug));
        }
        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get download info for a module binary.
    pub async fn get_download_info(
        &self,
        username: &str,
        slug: &str,
        platform: &str,
    ) -> Result<DownloadInfo, String> {
        let url = format!(
            "{}/modules/@{}/{}/download/{}",
            self.base_url, username, slug, platform
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Download failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse download info: {}", e))
    }

    /// Download a module binary archive and return the bytes.
    pub async fn download_binary(&self, download_url: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .http
            .get(download_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download binary: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Download returned HTTP {}", resp.status()));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read binary data: {}", e))
    }

    /// List files available for a module on StarkHub.
    pub async fn list_module_files(
        &self,
        username: &str,
        slug: &str,
    ) -> Result<Vec<ModuleFileSummary>, String> {
        let url = format!(
            "{}/modules/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list module files: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to list files: {}", body));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse file list: {}", e))?;

        let files = json
            .get("files")
            .and_then(|v| serde_json::from_value::<Vec<ModuleFileSummary>>(v.clone()).ok())
            .unwrap_or_default();

        Ok(files)
    }

    /// Download a single file from a module on StarkHub.
    pub async fn download_module_file(
        &self,
        username: &str,
        slug: &str,
        file_name: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{}/modules/@{}/{}/files/{}",
            self.base_url, username, slug, file_name
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download file '{}': {}", file_name, e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to download '{}': {}", file_name, body));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse file response: {}", e))?;

        json.get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("File '{}' response missing content field", file_name))
    }

    // --- Agent Subtype methods ---

    /// List agent subtypes from StarkHub.
    pub async fn list_agent_subtypes(&self) -> Result<Vec<AgentSubtypeSummary>, String> {
        let url = format!("{}/agent-subtypes", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        let paginated: PaginatedResponse<AgentSubtypeSummary> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(paginated.data)
    }

    /// Get agent subtype detail by @username/slug.
    pub async fn get_agent_subtype(
        &self,
        username: &str,
        slug: &str,
    ) -> Result<AgentSubtypeDetail, String> {
        let url = format!("{}/agent-subtypes/@{}/{}", self.base_url, username, slug);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if resp.status().as_u16() == 404 {
            return Err(format!(
                "Agent subtype @{}/{} not found on StarkHub",
                username, slug
            ));
        }
        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Download the raw agent.md content for an agent subtype.
    pub async fn download_agent_subtype(
        &self,
        username: &str,
        slug: &str,
        auth_token: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{}/agent-subtypes/@{}/{}/download",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Download failed: {}", body));
        }

        resp.text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))
    }

    /// List files for an agent subtype.
    pub async fn list_agent_subtype_files(
        &self,
        username: &str,
        slug: &str,
    ) -> Result<Vec<AgentSubtypeFileSummary>, String> {
        let url = format!(
            "{}/agent-subtypes/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let files: Vec<AgentSubtypeFileSummary> =
            serde_json::from_value(result["files"].clone()).unwrap_or_default();

        Ok(files)
    }

    /// Get a single file's content for an agent subtype.
    pub async fn get_agent_subtype_file(
        &self,
        username: &str,
        slug: &str,
        filename: &str,
    ) -> Result<AgentSubtypeFileDetail, String> {
        let url = format!(
            "{}/agent-subtypes/@{}/{}/files/{}",
            self.base_url, username, slug, filename
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to get file: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Publish an agent subtype (agent.md) to StarkHub.
    pub async fn publish_agent_subtype(
        &self,
        raw_agent_md: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/agent-subtypes", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "raw_agent_md": raw_agent_md }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Publish failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Upload a file for an agent subtype.
    pub async fn upload_agent_subtype_file(
        &self,
        username: &str,
        slug: &str,
        file_name: &str,
        content: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/agent-subtypes/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "file_name": file_name,
                "content": content,
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("File upload failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Publish a module manifest to StarkHub.
    pub async fn publish_module(
        &self,
        manifest_toml: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/modules", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "manifest_toml": manifest_toml }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Publish failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Upload a file for a module on StarkHub.
    pub async fn upload_module_file(
        &self,
        username: &str,
        slug: &str,
        file_name: &str,
        content: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/modules/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "file_name": file_name,
                "content": content,
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("File upload failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // --- Skill methods ---

    /// Get featured skills from StarkHub.
    pub async fn get_featured_skills(&self) -> Result<Vec<SkillSummary>, String> {
        let url = format!("{}/skills/featured", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Search skills on StarkHub.
    pub async fn search_skills(&self, query: &str) -> Result<Vec<SkillSummary>, String> {
        let url = format!("{}/search", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        let paginated: PaginatedResponse<SkillSummary> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(paginated.data)
    }

    /// Get skill detail by @username/slug.
    pub async fn get_skill(
        &self,
        username: &str,
        slug: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/skills/@{}/{}", self.base_url, username, slug);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if resp.status().as_u16() == 404 {
            return Err(format!("Skill @{}/{} not found on StarkHub", username, slug));
        }
        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Publish a skill to StarkHub.
    pub async fn publish_skill(
        &self,
        raw_markdown: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/submit", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "raw_markdown": raw_markdown }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Publish failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// List files for a skill on StarkHub.
    pub async fn list_skill_files(
        &self,
        username: &str,
        slug: &str,
    ) -> Result<Vec<SkillFileSummary>, String> {
        let url = format!(
            "{}/skills/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("StarkHub returned HTTP {}", resp.status()));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let files: Vec<SkillFileSummary> =
            serde_json::from_value(result["files"].clone()).unwrap_or_default();

        Ok(files)
    }

    /// Get a single file's content for a skill on StarkHub.
    pub async fn get_skill_file(
        &self,
        username: &str,
        slug: &str,
        filename: &str,
    ) -> Result<SkillFileDetail, String> {
        let url = format!(
            "{}/skills/@{}/{}/files/{}",
            self.base_url, username, slug, filename
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to get file: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Upload a file for a skill on StarkHub.
    pub async fn upload_skill_file(
        &self,
        username: &str,
        slug: &str,
        file_name: &str,
        content: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/skills/@{}/{}/files",
            self.base_url, username, slug
        );
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "file_name": file_name,
                "content": content,
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to StarkHub: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("File upload failed: {}", body));
        }

        resp.json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

/// Detect the current platform string for binary downloads.
pub fn current_platform() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "linux-x86_64" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "linux-aarch64" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "darwin-x86_64" }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "darwin-aarch64" }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
    )))]
    { "unknown" }
}
