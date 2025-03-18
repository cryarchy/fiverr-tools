use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum TitleError {
    #[error("TitleError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

impl GigPage {
    pub fn get_title(&self) -> Result<String, TitleError> {
        let title_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main > .gig-overview > h1".to_owned(),
        );
        let title = self
            .tab
            .find_element(title_selector.as_ref())
            .map_err(Self::map_err_fn(title_selector.to_owned()))?
            .get_inner_text()
            .map_err(Self::map_err_fn(title_selector))?;
        Ok(title)
    }
}
