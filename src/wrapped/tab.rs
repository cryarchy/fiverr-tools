use std::{sync::Arc, time::Duration};

use headless_chrome::Tab;

use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::{Error, WrappedElement};

#[derive(Clone)]
pub struct WrappedTab {
    tab: Arc<Tab>,
}

impl WrappedTab {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    pub fn get_title(&self) -> Result<String, Error> {
        self.tab.get_title().map_err(Error::GetTitle)
    }

    pub fn find_element<'a>(&'a self, selector: &Selector) -> Result<WrappedElement<'a>, Error> {
        self.tab
            .find_element(selector)
            .map(|el| WrappedElement::new(el, selector.to_owned()))
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn find_elements<'a>(
        &'a self,
        selector: &Selector,
        auto_index: bool,
    ) -> Result<Vec<WrappedElement<'a>>, Error> {
        self.tab
            .find_elements(selector)
            .map(|els| {
                els.into_iter()
                    .enumerate()
                    .map(|(i, el)| {
                        let selector = match auto_index {
                            true => selector.nth_child(i + 1),
                            false => selector.to_owned(),
                        };
                        WrappedElement::new(el, selector)
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn wait_for_element<'a>(
        &'a self,
        selector: &Selector,
    ) -> Result<WrappedElement<'a>, Error> {
        self.tab
            .wait_for_element(selector)
            .map(|el| WrappedElement::new(el, selector.to_owned()))
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn wait_for_element_with_custom_timeout<'a>(
        &'a self,
        selector: &Selector,
        timeout: Duration,
    ) -> Result<WrappedElement<'a>, Error> {
        self.tab
            .wait_for_element_with_custom_timeout(selector, timeout)
            .map(|el| WrappedElement::new(el, selector.to_owned()))
            .map_err(|e| MarkupInteractionError::new(e, selector.to_string()).into())
    }

    pub fn wait_until_navigated(&self) -> Result<(), Error> {
        self.tab
            .wait_until_navigated()
            .map(|_| ())
            .map_err(Error::WaitUntilNavigated)
    }

    pub fn navigate_to(&self, url: &str) -> Result<(), Error> {
        self.tab
            .navigate_to(url)
            .map(|_| ())
            .map_err(|e| Error::NavigatedTo(url.to_owned(), e))?;
        self.wait_until_navigated()
    }
}
