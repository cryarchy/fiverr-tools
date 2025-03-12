use crate::selector::Selector;

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum GigMetadataError {
    #[error("GigMetadataError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

#[derive(Debug)]
pub struct GigMetadata {
    name: String,
    values: Vec<String>,
}

impl GigDataExtractor {
    pub fn extract_gig_metadata(&self) -> Result<Vec<GigMetadata>, GigMetadataError> {
        let metadata_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-description > .metadata > .metadata-attribute".to_owned());
        let metadata_els = self
            .tab
            .find_elements(metadata_els_selector.as_ref())
            .map_err(|e| MarkupInteractionError::new(e, metadata_els_selector.to_string()))?;
        let mut gig_metadata = Vec::new();
        for (i, metadata_el) in metadata_els.into_iter().enumerate() {
            let get_child_selector =
                |child_selector: &str| metadata_els_selector.nth_child(i).append(child_selector);

            let name_selector = "p";
            let name = metadata_el
                .find_element("p")
                .map_err(Self::map_err_fn(get_child_selector(name_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(name_selector)))?;
            let mut values = Vec::new();
            let relative_values_selector = "li";
            let value_els = metadata_el
                .find_elements(relative_values_selector)
                .map_err(Self::map_err_fn(get_child_selector(
                    relative_values_selector,
                )))?;
            for (y, value_el) in value_els.into_iter().enumerate() {
                values.push(value_el.get_inner_text().map_err(|e| {
                    let absolute_value_el_selector = metadata_els_selector
                        .nth_child(i)
                        .append(relative_values_selector)
                        .nth_child(y);
                    MarkupInteractionError::new(e, absolute_value_el_selector.to_string())
                })?);
            }
            gig_metadata.push(GigMetadata { name, values });
        }
        Ok(gig_metadata)
    }
}
