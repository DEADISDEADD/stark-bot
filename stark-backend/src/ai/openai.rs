use crate::ai::Message;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct OpenAICompletionRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAICompletionResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorResponse {
    error: OpenAIError,
}

#[derive(Debug, Deserialize)]
struct OpenAIError {
    message: String,
}

impl OpenAIClient {
    pub fn new(api_key: &str, endpoint: Option<&str>, model: Option<&str>) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let auth_value = header::HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| format!("Invalid API key format: {}", e))?;
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            endpoint: endpoint
                .unwrap_or("https://api.openai.com/v1/chat/completions")
                .to_string(),
            model: model.unwrap_or("gpt-4o").to_string(),
        })
    }

    pub async fn generate_text(&self, messages: Vec<Message>) -> Result<String, String> {
        let api_messages: Vec<OpenAIMessage> = messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: m.role.to_string(),
                content: m.content,
            })
            .collect();

        let request = OpenAICompletionRequest {
            model: self.model.clone(),
            messages: api_messages,
            max_tokens: 4096,
        };

        log::debug!("Sending request to OpenAI API: {:?}", request);

        let response = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<OpenAIErrorResponse>(&error_text) {
                return Err(format!("OpenAI API error: {}", error_response.error.message));
            }

            return Err(format!(
                "OpenAI API returned error status: {}, body: {}",
                status, error_text
            ));
        }

        let response_data: OpenAICompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        response_data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "OpenAI API returned no content".to_string())
    }
}
