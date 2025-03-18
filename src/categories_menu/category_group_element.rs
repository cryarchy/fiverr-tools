use std::sync::Arc;

use headless_chrome::Tab;

use crate::{
    selector::Selector,
    wrapped_element::{WrappedElement, WrappedElementError},
};

use super::error::CategoriesMenuError;

pub struct CategoryGroupElement<'a> {
    tab: &'a Arc<Tab>,
    element: WrappedElement<'a>,
}

impl<'a> CategoryGroupElement<'a> {
    pub(super) fn new(tab: &'a Arc<Tab>, element: WrappedElement<'a>) -> Self {
        Self { tab, element }
    }

    pub fn selector(&self) -> &Selector {
        self.element.selector()
    }

    fn name_el_selector(&self) -> Selector {
        self.selector()
            .append(".linked-title:first-child :first-child")
    }

    pub fn name(&self) -> Result<String, CategoriesMenuError> {
        let selector = self.name_el_selector();
        self.tab
            .find_element(selector.as_ref())
            .map_err(|e| WrappedElementError::markup_interaction(e, &selector))?
            .get_inner_text()
            .map_err(|e| WrappedElementError::markup_interaction(e, &selector).into())
    }
}
