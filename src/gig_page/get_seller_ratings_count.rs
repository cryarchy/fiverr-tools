use crate::{selector::Selector, string_cleaner::STRING_CLEANER};

use super::GigPage;

impl GigPage {
    pub fn get_seller_ratings_count(&self) -> Result<usize, crate::Error> {
        let ratings_count_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .ratings-count > span"
                .to_owned(),
        );
        let ratings_count = self
            .tab
            .find_element(&ratings_count_selector)?
            .get_inner_text()?;
        STRING_CLEANER
            .as_usize(&ratings_count)
            .map_err(|e| e.into())
    }
}
