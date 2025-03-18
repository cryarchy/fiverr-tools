use crate::{
    markup_interaction_error::MarkupInteractionError,
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum SellerRatingsCountError {
    #[error("SellerRatingsCountError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("SellerRatingsCountError: {0}")]
    StringCleaner(#[from] StringCleanerError),
}

impl GigPage {
    pub fn get_seller_ratings_count(&self) -> Result<usize, SellerRatingsCountError> {
        let ratings_count_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .ratings-count > span"
                .to_owned(),
        );
        let ratings_count = self
            .tab
            .find_element(ratings_count_selector.as_ref())
            .map_err(Self::map_err_fn(ratings_count_selector.to_owned()))?
            .get_inner_text()
            .map_err(Self::map_err_fn(ratings_count_selector))?;
        STRING_CLEANER
            .as_usize(&ratings_count)
            .map_err(|e| e.into())
    }
}
