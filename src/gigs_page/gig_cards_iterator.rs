use std::sync::Arc;

use headless_chrome::Tab;

use crate::{
    markup_interaction_error::MarkupInteractionError, selector::Selector,
    string_cleaner::STRING_CLEANER, wrapped::Error,
};

#[derive(Debug)]
pub struct GigCard {
    pub url: String,
    pub page: usize,
}

pub struct GigCardsIterator<'a> {
    tab: &'a Arc<Tab>,
    current_index: usize,
    page: usize,
    minimum_rating: usize,
}

impl<'a> GigCardsIterator<'a> {
    pub(super) fn new(tab: &'a Arc<Tab>, page: usize, minimum_rating: usize) -> Self {
        Self {
            tab,
            current_index: 1,
            page,
            minimum_rating,
        }
    }

    fn selector() -> Selector {
        Selector::new(
            "#main-wrapper .listings-perseus .listing-container .gig-card-layout".to_owned(),
        )
    }

    fn _next(&mut self) -> Result<Option<GigCard>, crate::Error> {
        loop {
            let selector = Self::selector().nth_child(self.current_index);
            let url_selector = selector.append(r#"a[aria-label="Go to gig"]"#);
            let gig_anchor = match self.tab.find_element(url_selector.as_ref()) {
                Ok(gig_anchor) => gig_anchor,
                Err(e) => {
                    println!(
                        "Expected iterator error: {}",
                        MarkupInteractionError::new(e, selector.to_string())
                    );
                    return Ok(None);
                }
            };
            let url = gig_anchor
                .get_attribute_value("href")
                .map_err(|e| MarkupInteractionError::new(e, selector.to_string()))?
                .ok_or(Error::AttributeNotFound(
                    "href".to_owned(),
                    selector.to_string(),
                ))?;
            let ratings_count_selector =
                selector.append(".orca-rating .ratings-count .rating-count-number");
            let ratings_count = self
                .tab
                .find_element(ratings_count_selector.as_ref())
                .map_err(|e| MarkupInteractionError::new(e, selector.to_string()))?
                .get_inner_text()
                .map_err(|e| MarkupInteractionError::new(e, selector.to_string()))?;

            if ratings_count.contains("k") {
                self.current_index += 1;
                return Ok(Some(GigCard {
                    url,
                    page: self.page,
                }));
            } else {
                let ratings_count = STRING_CLEANER.as_usize(&ratings_count)?;
                self.current_index += 1;
                if ratings_count > self.minimum_rating {
                    return Ok(Some(GigCard {
                        url,
                        page: self.page,
                    }));
                }
            }
        }
    }
}

impl Iterator for GigCardsIterator<'_> {
    type Item = Result<GigCard, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(sub_category) => sub_category.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
