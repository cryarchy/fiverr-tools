use crate::selector::Selector;

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum GigRatingError {
    #[error("GigRatingError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
}

impl GigDataExtractor {
    pub fn extract_gig_rating(&self) -> Result<String, GigRatingError> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-overview > .seller-overview div:has(button) > strong".to_owned());
        let rating_el = self
            .tab
            .find_element(selector.as_ref())
            .map_err(Self::map_err_fn(selector.to_owned()))?;
        let rating = rating_el
            .get_inner_text()
            .map_err(Self::map_err_fn(selector))?;
        Ok(rating)
    }
}
