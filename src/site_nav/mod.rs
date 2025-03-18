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

    fn get_home_button(&self) -> Result<WrappedElement, crate::Error> {
        let home_button_selector = Self::home_button_selector();
        self.tab
            .find_element(&home_button_selector)
            .map_err(|e| e.into())
    }

    pub fn go_home(&self) -> Result<(), crate::Error> {
        self.get_home_button()?
            .click()
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
