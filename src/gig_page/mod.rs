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

use crate::wrapped::WrappedTab;

pub struct GigPage {
    tab: WrappedTab,
}

impl GigPage {
    pub fn new(tab: WrappedTab) -> Self {
        Self { tab }
    }
}
