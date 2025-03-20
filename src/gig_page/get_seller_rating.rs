use crate::selector::Selector;

use super::GigPage;

impl GigPage {
    pub fn get_seller_rating(&self) -> Result<String, crate::Error> {
        let ratings_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .rating-score".to_owned(),
        );
        let rating = self.tab.find_element(&ratings_selector)?.get_inner_text()?;
        Ok(rating)
    }
}
