use std::sync::Arc;

use headless_chrome::Tab;

use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::{Error, WrappedElement};

pub struct WrappedTab {
    tab: Arc<Tab>,
}

impl WrappedTab {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    pub fn find_element<'a>(&'a self, selector: &Selector) -> Result<WrappedElement<'a>, Error> {
        self.tab
            .find_element(&selector)
            .map(|el| WrappedElement::new(el, selector.to_owned()))
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn find_elements<'a>(
        &'a self,
        selector: &Selector,
    ) -> Result<Vec<WrappedElement<'a>>, Error> {
        self.tab
            .find_elements(&selector)
            .map(|els| {
                els.into_iter()
                    .map(|el| WrappedElement::new(el, selector.to_owned()))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn wait_for_element<'a>(
        &'a self,
        selector: &Selector,
    ) -> Result<WrappedElement<'a>, Error> {
        self.tab
            .wait_for_element(&selector)
            .map(|el| WrappedElement::new(el, selector.to_owned()))
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }
}
