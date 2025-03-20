use crate::{
    markup_interaction_error::MarkupInteractionError, selector::Selector,
    string_cleaner::StringCleanerError, wrapped,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected: {0}")]
    Unexpected(String),
    #[error("WrappedElementError: {0}")]
    WrappedElement(#[from] wrapped::Error),
    #[error("GigsPageError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("StringCleanerError: {0}")]
    StringCleaner(#[from] StringCleanerError),
    #[error("ElementNotFound: {0}")]
    ElementNotFound(Selector),
}
