use crate::selector::Selector;

use super::GigPage;

impl GigPage {
    pub fn get_seller_description(&self) -> Result<String, crate::Error> {
        let description_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .seller-desc > .inner"
                .to_owned(),
        );
        let description = self
            .tab
            .find_element(&description_selector)?
            .get_inner_text()?;
        Ok(description)
    }
}
