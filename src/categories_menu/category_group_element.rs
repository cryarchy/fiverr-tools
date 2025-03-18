use crate::{
    selector::Selector,
    wrapped::{WrappedElement, WrappedTab},
};

pub struct CategoryGroupElement<'a> {
    tab: &'a WrappedTab,
    element: WrappedElement<'a>,
}

impl<'a> CategoryGroupElement<'a> {
    pub(super) fn new(tab: &'a WrappedTab, element: WrappedElement<'a>) -> Self {
        Self { tab, element }
    }

    pub fn selector(&self) -> &Selector {
        self.element.selector()
    }

    fn name_el_selector(&self) -> Selector {
        self.selector()
            .append(".linked-title:first-child :first-child")
    }

    pub fn name(&self) -> Result<String, crate::Error> {
        let selector = self.name_el_selector();
        self.tab
            .find_element(&selector)?
            .get_inner_text()
            .map_err(|e| e.into())
    }
}
