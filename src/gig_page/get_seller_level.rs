use crate::{selector::Selector, string_cleaner::STRING_CLEANER};

use super::GigPage;

impl GigPage {
    pub fn get_seller_level(&self) -> Result<String, crate::Error> {
        let seller_level_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .seller-card div:has(.rating-score) div:has(p) > p".to_owned());
        let seller_level_els = self.tab.find_elements(&seller_level_selector)?;
        let Some(seller_level_el) = seller_level_els.last() else {
            return Err(crate::Error::Unexpected(
                "Seller lever element not found!".to_owned(),
            ));
        };
        let seller_level = seller_level_el.get_inner_text()?;
        STRING_CLEANER
            .as_simple_text(&seller_level)
            .map_err(|e| e.into())
    }
}
