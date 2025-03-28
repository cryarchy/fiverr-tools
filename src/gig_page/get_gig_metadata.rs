use crate::selector::Selector;

use super::GigPage;

#[derive(Debug)]
pub struct GigMetadata {
    pub name: String,
    pub values: Vec<String>,
}

impl GigPage {
    pub fn get_gig_metadata(&self) -> Result<Vec<GigMetadata>, crate::Error> {
        let metadata_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main > .gig-description > .metadata > .metadata-attribute".to_owned());
        let metadata_els = self.tab.find_elements(&metadata_els_selector, true)?;
        let mut gig_metadata = Vec::new();
        for metadata_el in metadata_els.into_iter() {
            let name_selector = metadata_el.selector().append("p");
            let name = self.tab.find_element(&name_selector)?.get_inner_text()?;
            let mut values = Vec::new();
            let relative_values_selector = metadata_el.selector().append("li");
            let value_els = self.tab.find_elements(&relative_values_selector, true)?;
            for value_el in value_els.into_iter() {
                values.push(value_el.get_inner_text()?);
            }
            gig_metadata.push(GigMetadata { name, values });
        }
        Ok(gig_metadata)
    }
}
