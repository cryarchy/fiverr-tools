use std::sync::LazyLock;

use regex::Regex;

pub static PRICE_RANGE_PARSER: LazyLock<PriceRangeParser> =
    LazyLock::new(|| PriceRangeParser::new().unwrap());

pub struct PriceRangeParser {
    price_range_re: Regex,
}

#[derive(Debug, thiserror::Error)]
pub enum PriceRangeParserError {
    #[error("{0}")]
    Regex(#[from] regex::Error),
    #[error("Invalid digit: {0}")]
    ParseInt(String),
    #[error("Extracting range's min val failed!")]
    MinVal,
    #[error("Extracting range's max val failed!")]
    MaxVal,
}

impl PriceRangeParser {
    pub fn new() -> Result<Self, PriceRangeParserError> {
        Ok(Self {
            price_range_re: Regex::new("[^0-9-]")?,
        })
    }

    pub fn get_range_tuple(&self, no: &str) -> Result<(usize, usize), PriceRangeParserError> {
        let price_range = self.price_range_re.replace_all(no, "");
        if !price_range.contains('-') {
            let max_val = price_range
                .parse()
                .map_err(|_| PriceRangeParserError::ParseInt(no.to_owned()))?;
            Ok((0, max_val))
        } else {
            let mut range_vals = price_range.split('-');
            let min_val = range_vals.next().ok_or(PriceRangeParserError::MinVal)?;
            let max_val = range_vals.next().ok_or(PriceRangeParserError::MaxVal)?;
            let min_val = min_val
                .parse()
                .map_err(|_| PriceRangeParserError::ParseInt(no.to_owned()))?;
            let max_val = max_val
                .parse()
                .map_err(|_| PriceRangeParserError::ParseInt(no.to_owned()))?;

            Ok((min_val, max_val))
        }
    }
}
