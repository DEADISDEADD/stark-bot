use crate::ai::Message;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Llama client for Ollama API
#[derive(Debug, Clone)]
pub struct LlamaClient {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaErrorResponse {
    error: String,
}

impl LlamaClient {
    pub fn new(endpoint: Option<&str>, model: Option<&str>) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(300)) // Llama can be slower
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            endpoint: endpoint
                .unwrap_or("http://localhost:11434/api/chat")
                .to_string(),
            model: model.unwrap_or("llama3.2").to_string(),
        })
    }

    pub async fn generate_text(&self, messages: Vec<Message>) -> Result<String, String> {
        let api_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role.to_string(),
                content: m.content,
            })
            .collect();

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: api_messages,
            stream: false,
        };

        log::debug!("Sending request to Ollama API: {:?}", request);

        let response = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Ollama API request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<OllamaErrorResponse>(&error_text) {
                return Err(format!("Ollama API error: {}", error_response.error));
            }

            return Err(format!(
                "Ollama API returned error status: {}, body: {}",
                status, error_text
            ));
        }

        let response_data: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        if response_data.message.content.is_empty() {
            return Err("Ollama API returned no content".to_string());
        }

        Ok(response_data.message.content)
    }
}
