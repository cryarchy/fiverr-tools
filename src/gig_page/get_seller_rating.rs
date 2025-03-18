use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum SellerRatingError {
    #[error("SellerRatingError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

impl GigPage {
    pub fn get_seller_rating(&self) -> Result<String, SellerRatingError> {
        let ratings_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .rating-score".to_owned(),
        );
        let rating = self
            .tab
            .find_element(ratings_selector.as_ref())
            .map_err(Self::map_err_fn(ratings_selector.to_owned()))?
            .get_inner_text()
            .map_err(Self::map_err_fn(ratings_selector))?;
        Ok(rating)
    }
}
