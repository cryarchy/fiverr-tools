use std::{ thread::sleep, time::Duration};


use crate::{selector::Selector, wrapped::WrappedTab};

use super::GigPage;

pub struct GigReviewIterator {
    tab: WrappedTab,
    current_index: usize,
}

impl GigReviewIterator {
    fn new(tab: WrappedTab) -> Result<Self, crate::Error> {
        Ok(Self {
            current_index: 1,
            tab,
        })
    }
}

impl Iterator for GigReviewIterator {
    type Item = Result<GigReview, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        fn _next(mut_self: &mut GigReviewIterator) -> Result<Option<GigReview>, crate::Error> {
            let gig_review_el_selector = Selector::new(
                "#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .review-item-component-wrapper".to_owned(),
                
            ).nth_child(mut_self.current_index);
            let gig_review_el = mut_self
                .tab
                .find_element(&gig_review_el_selector);
            let Ok(gig_review_el) = gig_review_el else {
                for _ in 0..3 {
                let show_more_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .reviews-wrap > div > button".to_owned());
                match mut_self.tab.find_element(&show_more_button_selector) {
                    Ok(show_more_button) => {
                        show_more_button
                            .click()?;
                        match mut_self.tab.wait_for_element(&gig_review_el_selector) {
                            Ok(_) => return _next(mut_self),
                            Err(e) => {
                                log::debug!("Error waiting for element: '{}' : {e}", gig_review_el_selector);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("Show more button find error: {e}");
                        return Ok(None);
                    },
                }
            }
                return Ok(None);
            };
            gig_review_el.scroll_into_view()?;
            let country_selector = gig_review_el.selector().append(".country p");
            let country = mut_self.tab
                .find_element(&country_selector)?
                .get_inner_text()?;

            let rating_selector = gig_review_el.selector().append("strong.rating-score");
            let rating = mut_self.tab
                .find_element(&rating_selector)?
                .get_inner_text()?;

            let show_more_button_selector = gig_review_el.selector().append(".expand-button button");
            let show_more_button_el = mut_self.tab.find_element(&show_more_button_selector);
            if let Ok(button) = show_more_button_el {
                button.click()?;
                sleep(Duration::from_millis(500));
            }

            let description_selector = gig_review_el_selector.append(".review-description p");
            let description = mut_self.tab
                .find_element(&description_selector)?
                .get_inner_text()?;

            let price_duration_els_selector = gig_review_el.selector().append(
                "div:has(p:nth-child(1)):has(p:nth-child(2):last-child) > p:first-child",
            );
            let mut price_duration_els = mut_self.tab
                .find_elements(&price_duration_els_selector)?
                .into_iter();
            let price = price_duration_els
                .next()
                .ok_or(crate::Error::ElementNotFound(price_duration_els_selector.nth_child(1)))?
                .get_inner_text()?;
            let duration = price_duration_els
                .next()
                .ok_or(crate::Error::ElementNotFound(price_duration_els_selector.nth_child(2)))?
                .get_inner_text()?;
            mut_self.current_index += 1;
            Ok(Some(GigReview {
                country,
                rating,
                description,
                price,
                duration,
            }))
        }
        match _next(self) {
            Ok(gig_faq) => gig_faq.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug)]
pub struct GigReview {
    pub country: String,
    pub rating: String,
    pub price: String,
    pub duration: String,
    pub description: String,
}

impl GigPage {
    pub fn get_gig_reviews(&self) -> Result<impl IntoIterator<Item = Result<GigReview, crate::Error>>, crate::Error> {
        GigReviewIterator::new(self.tab.clone())
    }
}
