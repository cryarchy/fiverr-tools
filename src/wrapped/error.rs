use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("'{0}' attribute not found for '{1}'")]
    AttributeNotFound(String, String),
    #[error("Error getting tab's title: {0}")]
    GetTitle(anyhow::Error),
    #[error("Error waiting for navigation: {0}")]
    WaitUntilNavigated(anyhow::Error),
    #[error("Error navigating to '{0}': {1}")]
    NavigatedTo(String, anyhow::Error),
    #[error("Expression evaluation error: {0}")]
    Evaluate(anyhow::Error),
}

impl Error {
    pub fn markup_interaction(e: anyhow::Error, s: &Selector) -> Self {
        Self::MarkupInteraction(MarkupInteractionError::new(e, s.to_string()))
    }
}
