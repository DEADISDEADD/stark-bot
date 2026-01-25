use crate::ai::{AiClient, Message, MessageRole};
use crate::channels::types::{DispatchResult, NormalizedMessage};
use crate::db::Database;
use crate::gateway::events::EventBroadcaster;
use crate::gateway::protocol::GatewayEvent;
use std::sync::Arc;

/// Dispatcher routes messages to the AI and returns responses
pub struct MessageDispatcher {
    db: Arc<Database>,
    broadcaster: Arc<EventBroadcaster>,
}

impl MessageDispatcher {
    pub fn new(db: Arc<Database>, broadcaster: Arc<EventBroadcaster>) -> Self {
        Self { db, broadcaster }
    }

    /// Dispatch a normalized message to the AI and return the response
    pub async fn dispatch(&self, message: NormalizedMessage) -> DispatchResult {
        // Emit message received event
        self.broadcaster.broadcast(GatewayEvent::channel_message(
            message.channel_id,
            &message.channel_type,
            &message.user_name,
            &message.text,
        ));

        // Get active agent settings from database
        let settings = match self.db.get_active_agent_settings() {
            Ok(Some(settings)) => settings,
            Ok(None) => {
                let error = "No AI provider configured. Please configure agent settings.".to_string();
                log::error!("{}", error);
                return DispatchResult::error(error);
            }
            Err(e) => {
                let error = format!("Database error: {}", e);
                log::error!("{}", error);
                return DispatchResult::error(error);
            }
        };

        log::info!(
            "Using {} provider with model {} for message dispatch",
            settings.provider,
            settings.model
        );

        // Create AI client from settings
        let client = match AiClient::from_settings(&settings) {
            Ok(c) => c,
            Err(e) => {
                let error = format!("Failed to create AI client: {}", e);
                log::error!("{}", error);
                return DispatchResult::error(error);
            }
        };

        // Build messages for the AI
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: format!(
                    "You are StarkBot, a helpful AI assistant. You are responding to a message from {} on {}. Keep responses concise and helpful.",
                    message.user_name, message.channel_type
                ),
            },
            Message {
                role: MessageRole::User,
                content: message.text.clone(),
            },
        ];

        // Generate response
        match client.generate_text(messages).await {
            Ok(response) => {
                // Emit response event
                self.broadcaster.broadcast(GatewayEvent::agent_response(
                    message.channel_id,
                    &message.user_name,
                    &response,
                ));

                log::info!(
                    "Generated response for {} on channel {} using {}",
                    message.user_name,
                    message.channel_id,
                    settings.provider
                );

                DispatchResult::success(response)
            }
            Err(e) => {
                let error = format!("AI generation error ({}): {}", settings.provider, e);
                log::error!("{}", error);
                DispatchResult::error(error)
            }
        }
    }
}
