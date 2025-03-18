mod get_gig_description;
mod get_gig_faqs;
mod get_gig_metadata;
mod get_gig_packages;
mod get_gig_rating;
mod get_gig_reviews;
mod get_gig_reviews_count;
mod get_gig_visuals;
mod get_seller_description;
mod get_seller_level;
mod get_seller_rating;
mod get_seller_ratings_count;
mod get_seller_stats;
mod get_title;

use std::sync::Arc;

use headless_chrome::Tab;

use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

pub struct GigPage {
    tab: Arc<Tab>,
}

impl GigPage {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    fn map_err_fn(selector: Selector) -> impl FnOnce(anyhow::Error) -> MarkupInteractionError {
        move |e: anyhow::Error| MarkupInteractionError::new(e, selector.to_string())
    }
}
