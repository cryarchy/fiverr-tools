use std::sync::LazyLock;

use regex::Regex;

pub static STRING_CLEANER: LazyLock<StringCleaner> =
    LazyLock::new(|| StringCleaner::new().unwrap());

pub struct StringCleaner {
    number_re: Regex,
    simple_text_re: Regex,
}

#[derive(Debug, thiserror::Error)]
pub enum StringCleanerError {
    #[error("{0}")]
    Regex(#[from] regex::Error),
    #[error("Invalid digit: {0}")]
    ParseInt(String),
}

impl StringCleaner {
    pub fn new() -> Result<Self, StringCleanerError> {
        Ok(Self {
            number_re: Regex::new("[^0-9]")?,
            simple_text_re: Regex::new(r#"[^0-9A-Za-z ]"#)?,
        })
    }

    pub fn as_usize(&self, no: &str) -> Result<usize, StringCleanerError> {
        let parsed_no = self
            .number_re
            .replace_all(no, "")
            .parse()
            .map_err(|_| StringCleanerError::ParseInt(no.to_owned()))?;
        Ok(parsed_no)
    }

    pub fn as_simple_text(&self, text: &str) -> Result<String, StringCleanerError> {
        let simple_text = self.simple_text_re.replace_all(text, "").trim().to_owned();
        Ok(simple_text)
    }
}
