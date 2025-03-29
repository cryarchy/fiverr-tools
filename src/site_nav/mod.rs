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

    fn home_url() -> &'static str {
        "https://www.fiverr.com/?source=top_nav"
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
        match self.get_home_button() {
            Ok(anchor_el) => {
                if let Err(e) = anchor_el.click() {
                    log::error!("{e}");
                    self.tab.navigate_to(Self::home_url())?;
                }
            }
            Err(e) => {
                log::error!("{e}");
                self.tab.navigate_to(Self::home_url())?;
            }
        }
        self.tab
            .wait_for_element_with_custom_timeout(
                &Self::home_card_selector(),
                Duration::from_secs(60),
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
