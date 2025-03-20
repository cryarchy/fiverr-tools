use crate::{selector::Selector, string_cleaner::STRING_CLEANER};

use super::GigPage;

impl GigPage {
    pub fn get_gig_reviews_count(&self) -> Result<usize, crate::Error> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-overview > .seller-overview div:has(button) > button".to_owned());
        let reviews_count = self.tab.find_element(&selector)?.get_inner_text()?;
        let reviews_count = STRING_CLEANER.as_usize(&reviews_count)?;
        Ok(reviews_count)
    }
}
