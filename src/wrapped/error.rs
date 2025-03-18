use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("WrappedElementError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("WrappedElementError: '{0}' attribute not found for '{1}'")]
    AttributeNotFound(String, String),
}

impl Error {
    pub fn markup_interaction(e: anyhow::Error, s: &Selector) -> Self {
        Self::MarkupInteraction(MarkupInteractionError::new(e, s.to_string()))
    }
}
