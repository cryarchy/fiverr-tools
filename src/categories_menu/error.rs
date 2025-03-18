use crate::wrapped_element::WrappedElementError;

#[derive(Debug, thiserror::Error)]
pub enum CategoriesMenuError {
    #[error("CategoriesMenuError: {0}")]
    WrappedElement(#[from] WrappedElementError),
    #[error("CategoriesMenuError: Unexpected: {0}")]
    Unexpected(String),
}
