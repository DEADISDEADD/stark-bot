pub mod claude;
pub mod llama;
pub mod openai;

pub use claude::ClaudeClient;
pub use llama::LlamaClient;
pub use openai::OpenAIClient;

use crate::models::{AgentSettings, AiProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl ToString for MessageRole {
    fn to_string(&self) -> String {
        match self {
            MessageRole::System => "system".to_string(),
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Unified AI client that works with any configured provider
pub enum AiClient {
    Claude(ClaudeClient),
    OpenAI(OpenAIClient),
    Llama(LlamaClient),
}

impl AiClient {
    /// Create an AI client from agent settings
    pub fn from_settings(settings: &AgentSettings) -> Result<Self, String> {
        let provider = settings.provider_enum().ok_or_else(|| {
            format!("Unknown provider: {}", settings.provider)
        })?;

        match provider {
            AiProvider::Claude => {
                let client = ClaudeClient::new(
                    &settings.api_key,
                    Some(&settings.endpoint),
                    Some(&settings.model),
                )?;
                Ok(AiClient::Claude(client))
            }
            AiProvider::OpenAI => {
                let client = OpenAIClient::new(
                    &settings.api_key,
                    Some(&settings.endpoint),
                    Some(&settings.model),
                )?;
                Ok(AiClient::OpenAI(client))
            }
            AiProvider::Llama => {
                let client = LlamaClient::new(
                    Some(&settings.endpoint),
                    Some(&settings.model),
                )?;
                Ok(AiClient::Llama(client))
            }
        }
    }

    /// Generate text using the configured provider
    pub async fn generate_text(&self, messages: Vec<Message>) -> Result<String, String> {
        match self {
            AiClient::Claude(client) => client.generate_text(messages).await,
            AiClient::OpenAI(client) => client.generate_text(messages).await,
            AiClient::Llama(client) => client.generate_text(messages).await,
        }
    }
}
