use crate::selector::Selector;

use super::GigPage;

impl GigPage {
    pub fn title_selector() -> Selector {
        Selector::new(
            "#main-wrapper > .main-content .gig-page > .main > .gig-overview > h1".to_owned(),
        )
    }

    pub fn get_title(&self) -> Result<String, crate::Error> {
        let title_selector = Self::title_selector();
        let title = self.tab.find_element(&title_selector)?.get_inner_text()?;
        Ok(title)
    }
}
