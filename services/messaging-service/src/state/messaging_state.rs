use std::sync::Arc;

use crate::application::{ConversationService, MessageService, ReactionService};

/// Messaging domain state — holds all application services.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug)]
pub struct MessagingState {
    pub message_service: Arc<MessageService>,
    pub reaction_service: Arc<ReactionService>,
    pub conversation_service: Arc<ConversationService>,
}

impl MessagingState {
    #[must_use]
    pub const fn new(
        message_service: Arc<MessageService>,
        reaction_service: Arc<ReactionService>,
        conversation_service: Arc<ConversationService>,
    ) -> Self {
        Self {
            message_service,
            reaction_service,
            conversation_service,
        }
    }
}
