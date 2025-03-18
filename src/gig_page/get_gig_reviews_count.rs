use crate::{
    markup_interaction_error::MarkupInteractionError,
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum GigReviewsCountError {
    #[error("GigPackagesError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
    #[error("GigPackagesError: {0}")]
    StringCleanerError(#[from] StringCleanerError),
}

impl GigPage {
    pub fn get_gig_reviews_count(&self) -> Result<usize, GigReviewsCountError> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-overview > .seller-overview div:has(button) > button".to_owned());
        let reviews_el = self
            .tab
            .find_element(selector.as_ref())
            .map_err(Self::map_err_fn(selector.to_owned()))?;
        let rating_text = reviews_el
            .get_inner_text()
            .map_err(Self::map_err_fn(selector))?;
        let reviews_count = STRING_CLEANER.as_usize(&rating_text)?;
        Ok(reviews_count)
    }
}
