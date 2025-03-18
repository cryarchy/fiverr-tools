use std::sync::Arc;

use headless_chrome::Tab;

use crate::{
    selector::Selector,
    wrapped_element::{WrappedElement, WrappedElementError},
};

use super::error::CategoriesMenuError;

pub struct MainCategoryElement<'a> {
    tab: &'a Arc<Tab>,
    element: WrappedElement<'a>,
}

impl<'a> MainCategoryElement<'a> {
    pub(super) fn new(tab: &'a Arc<Tab>, element: WrappedElement<'a>) -> Self {
        Self { tab, element }
    }

    pub fn selector(&self) -> &Selector {
        self.element.selector()
    }

    pub fn menu_panel_selector(&self) -> Selector {
        self.element.selector().append(".menu-panel")
    }

    fn anchor_selector(&self) -> Selector {
        self.element.selector().append("a")
    }

    pub fn name(&self) -> Result<String, CategoriesMenuError> {
        let selector = self.anchor_selector();
        self.tab
            .find_element(selector.as_ref())
            .map_err(|e| WrappedElementError::markup_interaction(e, &selector))?
            .get_inner_text()
            .map_err(|e| WrappedElementError::markup_interaction(e, &selector).into())
    }
}
