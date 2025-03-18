use std::{sync::Arc, time::Duration};

use headless_chrome::Tab;

use crate::{
    markup_interaction_error::MarkupInteractionError,
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

#[derive(Debug)]
pub struct GigCard {
    pub url: String,
    pub page: usize,
}

pub struct GigsPage {
    tab: Arc<Tab>,
}

impl GigsPage {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    fn gig_els_selector() -> Selector {
        Selector::new(
            "#main-wrapper .listings-perseus .listing-container .gig-card-layout".to_owned(),
        )
    }

    // pub fn get_gigs(&self) -> Result<Vec<GigCard>, GigsPageError> {
    //     let selector = Self::gig_els_selector();
    //     let gig_card_els = self.tab.find_elements(selector.as_ref()).map_err(|e| MarkupInteractionError::new(e, &selector))?;
    //     let mut gig_cards = Vec::with_capacity(gig_card_els.len());
    //     for (i, gig_card_el) in gig_card_els.into_iter().enumerate() {
    //         let url_selector = selector.nth_child(i + 1).append(r#"a[aria-label="Go to gig"]"#);
    //         let gig_anchor = self.tab.find_element(url_selector.as_ref()).map_err(|e| MarkupInteractionError::new(e, &selector))?;

    //     }
    // }

    fn page_number_els_selector() -> Selector {
        Selector::new(
            r#"#main-wrapper .listings-perseus div:has([aria-label="Previous"]) > div > a:not([aria-label="Next"]):not([aria-label="Previous"])"#.to_owned(),
        )
    }

    pub fn go_to_page(&self, target_page_number: usize) -> Result<bool, GigsPageError> {
        let page_number_els_selector = Self::page_number_els_selector();
        let mut last_max_page_number = 0;
        loop {
            let page_number_els = self
                .tab
                .find_elements(page_number_els_selector.as_ref())
                .map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
            let first_pagination_el = page_number_els.first().ok_or(GigsPageError::Unexpected(
                "Empty pagination component".to_string(),
            ))?;
            let page_number = first_pagination_el
                .find_element("p")
                .map_err(|e| MarkupInteractionError::new(e, page_number_els_selector.to_string()))?
                .get_inner_text()
                .map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
            let page_number = STRING_CLEANER.as_usize(&page_number)?;
            if page_number > target_page_number {
                first_pagination_el.click().map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
                self.tab.wait_until_navigated().map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
                self.tab
                    .wait_for_element_with_custom_timeout(
                        page_number_els_selector.as_ref(),
                        Duration::from_secs(30),
                    )
                    .map_err(|e| {
                        MarkupInteractionError::new(e, page_number_els_selector.to_string())
                    })?;
                continue;
            }

            let last_pagination_el = page_number_els.last().ok_or(GigsPageError::Unexpected(
                "Empty pagination component".to_string(),
            ))?;
            let page_number = last_pagination_el
                .find_element("p")
                .map_err(|e| MarkupInteractionError::new(e, page_number_els_selector.to_string()))?
                .get_inner_text()
                .map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
            let page_number = STRING_CLEANER.as_usize(&page_number)?;
            if page_number < target_page_number {
                last_pagination_el.click().map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
                self.tab.wait_until_navigated().map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
                self.tab
                    .wait_for_element_with_custom_timeout(
                        page_number_els_selector.as_ref(),
                        Duration::from_secs(30),
                    )
                    .map_err(|e| {
                        MarkupInteractionError::new(e, page_number_els_selector.to_string())
                    })?;
                continue;
            }
            match page_number == last_max_page_number {
                true => break Ok(false),
                false => last_max_page_number = page_number,
            }

            let mut target_el = None;

            for page_number_el in page_number_els.into_iter() {
                let page_number = page_number_el
                    .find_element("p")
                    .map_err(|e| {
                        MarkupInteractionError::new(e, page_number_els_selector.to_string())
                    })?
                    .get_inner_text()
                    .map_err(|e| {
                        MarkupInteractionError::new(e, page_number_els_selector.to_string())
                    })?;
                let page_number = STRING_CLEANER.as_usize(&page_number)?;
                if page_number == target_page_number {
                    target_el = Some(page_number_el);
                }
            }

            target_el
                .ok_or(GigsPageError::Unexpected(
                    "Expected target page({target_page_number}) to be in pagination component"
                        .to_string(),
                ))?
                .click()
                .map_err(|e| {
                    MarkupInteractionError::new(e, page_number_els_selector.to_string())
                })?;
            self.tab.wait_until_navigated().map_err(|e| {
                MarkupInteractionError::new(e, page_number_els_selector.to_string())
            })?;
            break Ok(true);
        }
    }
}

pub struct GigsPagePagination<'a> {
    tab: &'a Arc<Tab>,
}

impl<'a> GigsPagePagination<'a> {
    pub fn new(tab: &'a Arc<Tab>) -> Self {
        Self { tab }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GigsPageError {
    #[error("GigsPageError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("GigsPageError: Unexpected: {0}")]
    Unexpected(String),
    #[error("GigsPageError: {0}")]
    StringCleaner(#[from] StringCleanerError),
}
