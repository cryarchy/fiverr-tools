use crate::{
    markup_interaction_error::MarkupInteractionError, selector::Selector,
    string_cleaner::STRING_CLEANER, wrapped::WrappedTab,
};

#[derive(Debug)]
pub struct GigCard {
    pub url: String,
}

pub struct GigCardsIterator<'a> {
    tab: &'a WrappedTab,
    current_index: usize,
    minimum_rating: usize,
}

impl<'a> GigCardsIterator<'a> {
    pub(super) fn new(tab: &'a WrappedTab, minimum_rating: usize) -> Self {
        Self {
            tab,
            current_index: 1,
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

            // Skip "Fiverr's Choice" gig cards
            let gig_card_text = self
                .tab
                .find_element(&selector)?
                .get_inner_text()?
                .to_lowercase();
            if gig_card_text.contains("fiverr") && gig_card_text.contains("choice") {
                self.current_index += 1;
                continue;
            }

            let url_selector = selector.append(r#"a[aria-label="Go to gig"]"#);
            let gig_anchor = match self.tab.find_element(&url_selector) {
                Ok(gig_anchor) => gig_anchor,
                Err(e) => {
                    log::debug!(
                        "Expected iterator error: {}",
                        MarkupInteractionError::new(e, selector.to_string())
                    );
                    return Ok(None);
                }
            };
            let url = gig_anchor.get_expected_attribute_value("href")?;
            let ratings_count_selector =
                selector.append(".orca-rating .ratings-count .rating-count-number");
            let Ok(ratings_count_el) = self.tab.find_element(&ratings_count_selector) else {
                // skip gig cards without ratings
                self.current_index += 1;
                continue;
            };
            let ratings_count = ratings_count_el.get_inner_text()?;

            if ratings_count.contains("k") {
                self.current_index += 1;
                return Ok(Some(GigCard { url }));
            } else {
                let ratings_count = STRING_CLEANER.as_usize(&ratings_count)?;
                self.current_index += 1;
                if ratings_count > self.minimum_rating {
                    return Ok(Some(GigCard { url }));
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
