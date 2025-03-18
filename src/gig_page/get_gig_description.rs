use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum GigDescriptionError {
    #[error("GigDescriptionError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

impl GigPage {
    pub fn get_gig_description(&self) -> Result<String, GigDescriptionError> {
        let selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-description > .description-wrapper > .description-content".to_owned());
        let description = self
            .tab
            .find_element(selector.as_ref())
            .map_err(Self::map_err_fn(selector.to_owned()))?
            .get_content()
            .map_err(Self::map_err_fn(selector))?;
        Ok(description)
    }
}
