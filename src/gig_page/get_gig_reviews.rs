use std::{sync::Arc, thread::sleep, time::Duration};

use headless_chrome::Tab;

use crate::{markup_interaction_error::MarkupInteractionError,selector::Selector};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum GigReviewsError {
    #[error("GigReviewsError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
    #[error("GigReviewsError: {0} element not found.")]
    ElementNotFound(String),
}

impl GigReviewsError {
    fn markup_interaction(e: anyhow::Error, selector: String) -> Self {
        Self::MarkupInteraction(MarkupInteractionError::new(e, selector))
    }

    fn map_err_fn(selector: Selector) -> impl FnOnce(anyhow::Error) -> Self {
        move |e: anyhow::Error| GigReviewsError::markup_interaction(e, selector.to_string())
    }
}

pub struct GigReviewIterator {
    tab: Arc<Tab>,
    current_index: usize,
}

impl GigReviewIterator {
    fn new(tab: Arc<Tab>) -> Result<Self, GigReviewsError> {
        Ok(Self {
            current_index: 1,
            tab,
        })
    }
}

impl Iterator for GigReviewIterator {
    type Item = Result<GigReview, GigReviewsError>;

    fn next(&mut self) -> Option<Self::Item> {
        fn _next(mut_self: &mut GigReviewIterator) -> Result<Option<GigReview>, GigReviewsError> {
            let gig_review_el_selector = Selector::new(
                "#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .review-item-component-wrapper".to_owned(),
                
            ).nth_child(mut_self.current_index);
            let gig_review_el = mut_self
                .tab
                .find_element(gig_review_el_selector.as_ref());
            let Ok(gig_review_el) = gig_review_el else {
                for _ in 0..3 {
                let show_more_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .reviews-wrap > div > button".to_owned());
                match mut_self.tab.find_element(show_more_button_selector.as_ref()) {
                    Ok(show_more_button) => {
                        show_more_button
                            .click()
                            .map_err(GigReviewsError::map_err_fn(show_more_button_selector))?;
                        match mut_self.tab.wait_for_element(gig_review_el_selector.as_ref()) {
                            Ok(_) => return _next(mut_self),
                            Err(e) => {
                                println!("Error waiting for element: '{}' : {e}", gig_review_el_selector);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Show more button find error: {e}");
                        return Ok(None);
                    },
                }
            }
                return Ok(None);
            };
            gig_review_el.scroll_into_view().map_err(GigReviewsError::map_err_fn(
                gig_review_el_selector.to_owned()
            ))?;
            let country_selector = ".country p";
            let country = gig_review_el
                .find_element(country_selector)
                .map_err(GigReviewsError::map_err_fn(
                    gig_review_el_selector.append(country_selector),
                ))?
                .get_inner_text()
                .map_err(GigReviewsError::map_err_fn(
                    gig_review_el_selector.append(country_selector),
                ))?;

            let rating_selector = "strong.rating-score";
            let rating = gig_review_el
                .find_element(rating_selector)
                .map_err(GigReviewsError::map_err_fn(
                    gig_review_el_selector.append(rating_selector),
                ))?
                .get_inner_text()
                .map_err(GigReviewsError::map_err_fn(
                    gig_review_el_selector.append(rating_selector),
                ))?;

            let show_more_button_selector = ".expand-button button";
            let show_more_button_el = gig_review_el.find_element(show_more_button_selector);
            if let Ok(button) = show_more_button_el {
                button.click().map_err(GigReviewsError::map_err_fn(
                    gig_review_el_selector.append(show_more_button_selector),
                ))?;
                sleep(Duration::from_millis(500));
            }

            let description_selector = gig_review_el_selector.append(".review-description p");
            let description = gig_review_el
                .find_element(description_selector.as_ref())
                .map_err(GigReviewsError::map_err_fn(description_selector.to_owned()))?
                .get_inner_text()
                .map_err(GigReviewsError::map_err_fn(description_selector))?;

            let price_duration_els_selector = Selector::new(
                "div:has(p:nth-child(1)):has(p:nth-child(2):last-child) > p:first-child".to_owned(),
            );
            let mut price_duration_els = gig_review_el
                .find_elements(price_duration_els_selector.as_ref())
                .map_err(GigReviewsError::map_err_fn(price_duration_els_selector.to_owned()))?
                .into_iter();
            let price = price_duration_els
                .next()
                .ok_or(GigReviewsError::ElementNotFound("price".to_owned()))?
                .get_inner_text()
                .map_err(GigReviewsError::map_err_fn(price_duration_els_selector.nth_child(1)))?;
            let duration = price_duration_els
                .next()
                .ok_or(GigReviewsError::ElementNotFound("duration".to_owned()))?
                .get_inner_text()
                .map_err(GigReviewsError::map_err_fn(price_duration_els_selector.nth_child(2)))?;
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
    country: String,
    rating: String,
    price: String,
    duration: String,
    description: String,
}

impl GigPage {
    pub fn get_gig_reviews(&self) -> Result<impl IntoIterator<Item = Result<GigReview, GigReviewsError>>, GigReviewsError> {
        GigReviewIterator::new(self.tab.clone())
    }
}
