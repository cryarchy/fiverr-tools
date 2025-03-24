use std::time::Duration;

use crate::{
    selector::Selector,
    wrapped::{WrappedElement, WrappedTab},
};

pub struct SiteNav {
    tab: WrappedTab,
}

impl SiteNav {
    pub fn new(tab: WrappedTab) -> Self {
        Self { tab }
    }

    fn home_button_selector() -> Selector {
        Selector::new("#Header a.site-logo".to_owned())
    }

    fn home_card_selector() -> Selector {
        Selector::new(".gig-card-layout .basic-gig-card a".to_owned())
    }

    fn get_home_button(&self) -> Result<WrappedElement, crate::Error> {
        let home_button_selector = Self::home_button_selector();
        self.tab
            .find_element(&home_button_selector)
            .map_err(|e| e.into())
    }

    pub fn go_home(&self) -> Result<(), crate::Error> {
        self.get_home_button()?.click()?;
        self.tab
            .wait_for_element_with_custom_timeout(
                &Self::home_card_selector(),
                Duration::from_secs(60),
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
