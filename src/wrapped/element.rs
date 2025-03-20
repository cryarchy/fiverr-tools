use headless_chrome::Element;

use crate::selector::Selector;

use super::Error;

pub struct WrappedElement<'a> {
    element: Element<'a>,
    selector: Selector,
}

impl<'a> WrappedElement<'a> {
    pub(super) fn new(element: Element<'a>, selector: Selector) -> Self {
        Self { element, selector }
    }

    pub fn get_content(&self) -> Result<String, Error> {
        self.element
            .get_content()
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn get_inner_text(&self) -> Result<String, Error> {
        self.element
            .get_inner_text()
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn get_attribute_value(&self, name: &str) -> Result<Option<String>, Error> {
        self.element
            .get_attribute_value(name)
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn get_expected_attribute_value(&self, name: &str) -> Result<String, Error> {
        self.get_attribute_value(name)?
            .ok_or(Error::AttributeNotFound(
                name.to_owned(),
                self.selector().to_string(),
            ))
    }

    pub fn click(&self) -> Result<&Self, Error> {
        self.element
            .click()
            .map(|_| self)
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn move_mouse_over(&self) -> Result<&Self, Error> {
        self.element
            .move_mouse_over()
            .map(|_| self)
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn scroll_into_view(&self) -> Result<&Self, Error> {
        self.element
            .scroll_into_view()
            .map(|_| self)
            .map_err(|e| Error::markup_interaction(e, self.selector()))
    }

    pub fn selector(&self) -> &Selector {
        &self.selector
    }
}
