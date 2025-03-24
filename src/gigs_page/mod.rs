mod gig_cards_iterator;

use std::time::Duration;

use gig_cards_iterator::GigCardsIterator;

use crate::{
    markup_interaction_error::MarkupInteractionError, selector::Selector,
    string_cleaner::STRING_CLEANER, wrapped::WrappedTab,
};

pub struct GigsPage {
    tab: WrappedTab,
    minimum_rating: usize,
}

impl GigsPage {
    pub fn new(tab: WrappedTab) -> Self {
        Self {
            tab,
            minimum_rating: 200,
        }
    }

    pub fn gigs(&self) -> Result<GigCardsIterator<'_>, crate::Error> {
        Ok(GigCardsIterator::new(
            &self.tab,
            self.get_current_page()?,
            self.minimum_rating,
        ))
    }

    fn current_page_selector() -> Selector {
        Selector::new(
            r#"#main-wrapper .listings-perseus div:has([aria-label="Previous"]) > div > a:not([aria-label="Next"]):not([aria-label="Previous"]):not([href])"#.to_owned(),
        )
    }

    pub fn get_current_page(&self) -> Result<usize, crate::Error> {
        let selector = Self::current_page_selector();
        let current_page = self.tab.find_element(&selector)?.get_inner_text()?;
        STRING_CLEANER.as_usize(&current_page).map_err(|e| e.into())
    }

    pub fn page_number_els_selector() -> Selector {
        Selector::new(
            r#"#main-wrapper .listings-perseus div:has([aria-label="Previous"]) > div > a:not([aria-label="Next"]):not([aria-label="Previous"])"#.to_owned(),
        )
    }

    fn get_page_number(&self, selector: &Selector) -> Result<usize, crate::Error> {
        let page_number = self.tab.find_element(selector)?.get_inner_text()?;
        STRING_CLEANER.as_usize(&page_number).map_err(|e| e.into())
    }

    pub fn go_to_page(&self, target_page_number: usize) -> Result<bool, crate::Error> {
        let page_number_els_selector = Self::page_number_els_selector();
        let mut last_max_page_number = 0;
        loop {
            self.tab.wait_for_element_with_custom_timeout(
                &page_number_els_selector,
                Duration::from_secs(300),
            )?;

            let current_page_number = self.get_current_page()?;
            log::debug!("Current page: {current_page_number} | Target page: {target_page_number}");
            if current_page_number == target_page_number {
                break Ok(true);
            }

            let page_number_els = self.tab.find_elements(&page_number_els_selector)?;
            let first_pagination_el = page_number_els.first().ok_or(crate::Error::Unexpected(
                "Empty pagination component".to_string(),
            ))?;
            let first_page_number_selector = first_pagination_el.selector().append("p");
            let first_page_number = self.get_page_number(&first_page_number_selector)?;
            log::debug!("First pagination number: {first_page_number}");
            if target_page_number < first_page_number {
                first_pagination_el.click()?;
                self.tab.wait_until_navigated()?;
                continue;
            }

            let last_pagination_el = page_number_els.last().ok_or(crate::Error::Unexpected(
                "Empty pagination component".to_string(),
            ))?;
            let last_page_number_selector = last_pagination_el.selector().append("p");
            let last_page_number = self.get_page_number(&last_page_number_selector)?;
            log::debug!("Last pagination number: {last_page_number}");
            match last_page_number == last_max_page_number {
                true => break Ok(false),
                false => last_max_page_number = last_page_number,
            }
            if target_page_number > last_page_number {
                last_pagination_el.click()?;
                self.tab.wait_until_navigated()?;
                continue;
            }

            let mut target_el = None;

            for page_number_el in page_number_els.into_iter() {
                let page_number_selector = page_number_el.selector().append("p");
                let page_number = self.get_page_number(&page_number_selector)?;
                if page_number == target_page_number {
                    target_el = Some(page_number_el);
                }
            }

            target_el
                .ok_or(crate::Error::Unexpected(
                    "Expected target page({target_page_number}) to be in pagination component"
                        .to_string(),
                ))?
                .click()?;
        }
    }
}
