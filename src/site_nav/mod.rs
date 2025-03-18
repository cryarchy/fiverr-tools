mod error;

use error::SiteNavError;
use headless_chrome::{Element, Tab};
use std::sync::Arc;

use crate::selector::Selector;

pub struct SiteNav {
    tab: Arc<Tab>,
}

impl SiteNav {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    fn home_button_selector() -> Selector {
        Selector::new("#Header a.site-logo".to_owned())
    }

    fn get_home_button(&self) -> Result<Element, SiteNavError> {
        let home_button_selector = Self::home_button_selector();
        self.tab
            .find_element(home_button_selector.as_ref())
            .map_err(|e| SiteNavError::markup_interaction(e, &home_button_selector))
    }

    pub fn go_home(&self) -> Result<(), SiteNavError> {
        self.get_home_button()?
            .click()
            .map(|_| ())
            .map_err(|e| SiteNavError::markup_interaction(e, &Self::home_button_selector()))
    }
}
