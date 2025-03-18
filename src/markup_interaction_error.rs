#[derive(Debug)]
pub struct MarkupInteractionError {
    selector: String,
    error: String,
}

impl MarkupInteractionError {
    pub fn new(error: impl ToString, selector: String) -> Self {
        Self {
            error: error.to_string(),
            selector,
        }
    }
}

impl std::fmt::Display for MarkupInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: '{}'", self.error, self.selector))
    }
}

impl std::error::Error for MarkupInteractionError {}
