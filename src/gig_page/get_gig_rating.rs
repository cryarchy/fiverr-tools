use crate::selector::Selector;

use super::GigPage;

impl GigPage {
    pub fn get_gig_rating(&self) -> Result<String, crate::Error> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-overview > .seller-overview div:has(button) > strong".to_owned());
        let rating_el = self.tab.find_element(&selector)?;
        let rating = rating_el.get_inner_text()?;
        Ok(rating)
    }
}
