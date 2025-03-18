use crate::{
    selector::Selector,
    wrapped::{WrappedElement, WrappedTab},
};

pub struct MainCategoryElement<'a> {
    tab: &'a WrappedTab,
    element: WrappedElement<'a>,
}

impl<'a> MainCategoryElement<'a> {
    pub(super) fn new(tab: &'a WrappedTab, element: WrappedElement<'a>) -> Self {
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

    pub fn name(&self) -> Result<String, crate::Error> {
        let selector = self.anchor_selector();
        self.tab
            .find_element(&selector)?
            .get_inner_text()
            .map_err(|e| e.into())
    }
}
