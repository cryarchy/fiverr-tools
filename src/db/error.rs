#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Unexpected: {0}")]
    Unexpected(String),
}
