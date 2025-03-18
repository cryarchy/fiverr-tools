mod error;

pub use error::WrappedElementError;
use headless_chrome::Element;

use crate::selector::Selector;

pub struct WrappedElement<'a> {
    element: Element<'a>,
    selector: Selector,
}

impl<'a> WrappedElement<'a> {
    pub fn new(element: Element<'a>, selector: Selector) -> Self {
        Self { element, selector }
    }

    pub fn get_inner_text(&self) -> Result<String, WrappedElementError> {
        self.element
            .get_inner_text()
            .map_err(|e| WrappedElementError::markup_interaction(e, self.selector()))
    }

    pub fn get_attribute_value(&self, name: &str) -> Result<Option<String>, WrappedElementError> {
        self.element
            .get_attribute_value(name)
            .map_err(|e| WrappedElementError::markup_interaction(e, self.selector()))
    }

    pub fn get_expected_attribute_value(&self, name: &str) -> Result<String, WrappedElementError> {
        self.get_attribute_value(name)?
            .ok_or(WrappedElementError::AttributeNotFound(
                name.to_owned(),
                self.selector().to_string(),
            ))
    }

    pub fn click(&self) -> Result<&Self, WrappedElementError> {
        self.element
            .click()
            .map(|_| self)
            .map_err(|e| WrappedElementError::markup_interaction(e, self.selector()))
    }

    pub fn move_mouse_over(&self) -> Result<&Self, WrappedElementError> {
        self.element
            .move_mouse_over()
            .map(|_| self)
            .map_err(|e| WrappedElementError::markup_interaction(e, self.selector()))
    }

    pub fn selector(&self) -> &Selector {
        &self.selector
    }
}
