use crate::selector::Selector;

use super::GigPage;

impl GigPage {
    pub fn get_gig_description(&self) -> Result<String, crate::Error> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-description > .description-wrapper > .description-content".to_owned());
        let description = self.tab.find_element(&selector)?.get_content()?;
        Ok(description)
    }
}
