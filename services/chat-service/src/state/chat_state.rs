use std::sync::Arc;

use crate::application::{MessageService, ReactionService};

/// Chat-domain services (messages and reactions).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct ChatState {
    pub message_service: Arc<MessageService>,
    pub reaction_service: Arc<ReactionService>,
}

impl ChatState {
    #[must_use]
    pub const fn new(
        message_service: Arc<MessageService>,
        reaction_service: Arc<ReactionService>,
    ) -> Self {
        Self {
            message_service,
            reaction_service,
        }
    }
}
