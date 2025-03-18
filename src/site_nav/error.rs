use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

#[derive(Debug, thiserror::Error)]
pub enum SiteNavError {
    #[error("SiteNavError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

impl SiteNavError {
    pub(super) fn markup_interaction(e: anyhow::Error, selector: &Selector) -> Self {
        Self::MarkupInteraction(MarkupInteractionError::new(e, selector.to_string()))
    }
}
