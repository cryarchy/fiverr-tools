use crate::{
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum SellerLevelError {
    #[error("SellerLevelError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("SellerLevelError: {0}")]
    StringCleaner(#[from] StringCleanerError),
    #[error("SellerLevelError: Unexpected '{0}'")]
    Unexpected(String),
}

impl GigDataExtractor {
    pub fn extract_seller_level(&self) -> Result<String, SellerLevelError> {
        let seller_level_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .seller-card div:has(.rating-score) div:has(p) > p".to_owned());
        let seller_level_els = self
            .tab
            .find_elements(seller_level_selector.as_ref())
            .map_err(Self::map_err_fn(seller_level_selector.to_owned()))?;
        let Some(seller_level_el) = seller_level_els.last() else {
            return Err(SellerLevelError::Unexpected(
                "Seller lever element not found!".to_owned(),
            ));
        };
        let seller_level = seller_level_el
            .get_inner_text()
            .map_err(Self::map_err_fn(seller_level_selector))?;
        STRING_CLEANER
            .as_simple_text(&seller_level)
            .map_err(|e| e.into())
    }
}
