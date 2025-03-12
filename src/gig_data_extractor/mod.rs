mod error;
mod extract_gig_description;
mod extract_gig_faqs;
mod extract_gig_metadata;
mod extract_gig_packages;
mod extract_gig_rating;
mod extract_gig_reviews;
mod extract_gig_reviews_count;
mod extract_gig_visuals;
mod extract_seller_description;
mod extract_seller_level;
mod extract_seller_rating;
mod extract_seller_ratings_count;
mod extract_seller_stats;
mod extract_title;

use std::sync::Arc;

use error::MarkupInteractionError;
use headless_chrome::Tab;

use crate::selector::Selector;

pub struct GigDataExtractor {
    tab: Arc<Tab>,
}

impl GigDataExtractor {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    fn map_err_fn(selector: Selector) -> impl FnOnce(anyhow::Error) -> MarkupInteractionError {
        move |e: anyhow::Error| MarkupInteractionError::new(e, selector.to_string())
    }
}
