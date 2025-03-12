use crate::{
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum SellerStatsError {
    #[error("SellerStatsError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("SellerStatsError: {0}")]
    StringCleaner(#[from] StringCleanerError),
}

#[derive(Debug)]
pub struct SellerStat {
    name: String,
    value: String,
}

impl GigDataExtractor {
    pub fn extract_seller_stats(&self) -> Result<Vec<SellerStat>, SellerStatsError> {
        let seller_stats_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .user-stats > li"
                .to_owned(),
        );
        let seller_stat_els = self
            .tab
            .find_elements(seller_stats_selector.as_ref())
            .map_err(Self::map_err_fn(seller_stats_selector.to_owned()))?;
        let mut seller_stats = Vec::new();

        for (i, seller_stat_el) in seller_stat_els.into_iter().enumerate() {
            let name = seller_stat_el
                .get_inner_text()
                .map_err(Self::map_err_fn(seller_stats_selector.nth_child(i + 1)))?;
            let value_selector = "strong";
            let value_selector_absolute = seller_stats_selector
                .nth_child(i + 1)
                .append(value_selector);
            let value = seller_stat_el
                .find_element(value_selector)
                .map_err(Self::map_err_fn(value_selector_absolute.to_owned()))?
                .get_inner_text()
                .map_err(Self::map_err_fn(value_selector_absolute))?;
            let name = STRING_CLEANER.as_simple_text(&name)?;
            let value = STRING_CLEANER.as_simple_text(&value)?;
            seller_stats.push(SellerStat { name, value });
        }
        Ok(seller_stats)
    }
}
