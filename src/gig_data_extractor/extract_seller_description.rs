use crate::selector::Selector;

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum SellerDescriptionError {
    #[error("SellerDescriptionError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
}

impl GigDataExtractor {
    pub fn extract_seller_description(&self) -> Result<String, SellerDescriptionError> {
        let description_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .seller-desc > .inner"
                .to_owned(),
        );
        let description = self
            .tab
            .find_element(description_selector.as_ref())
            .map_err(Self::map_err_fn(description_selector.to_owned()))?
            .get_inner_text()
            .map_err(Self::map_err_fn(description_selector.to_owned()))?;
        Ok(description)
    }
}
