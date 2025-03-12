use std::{thread::sleep, time::Duration};

use crate::selector::Selector;

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum GigReviewsError {
    #[error("GigReviewsError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
    #[error("GigReviewError: {0} element not found.")]
    ElementNotFound(String),
}

#[derive(Debug)]
pub struct GigReview {
    country: String,
    rating: String,
    price: String,
    duration: String,
    description: String,
}

impl GigDataExtractor {
    pub fn extract_gig_reviews(&self) -> Result<Vec<GigReview>, GigReviewsError> {
        let mut gig_reviews = Vec::new();
        loop {
            let element_index = gig_reviews.len() + 1;
            let gig_review_el_selector = Selector::new(format!(
                "#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .review-item-component-wrapper:nth-child({})",
                element_index
            ));
            let gig_review_el = self
                .tab
                .find_element(gig_review_el_selector.as_ref())
                .map_err(Self::map_err_fn(gig_review_el_selector.to_owned()));
            let Ok(gig_review_el) = gig_review_el else {
                let show_more_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-reviews .reviews-wrap > div > button".to_owned());
                let show_more_button = self.tab.find_element(show_more_button_selector.as_ref());
                match show_more_button {
                    Ok(show_more_button) => {
                        show_more_button
                            .click()
                            .map_err(Self::map_err_fn(show_more_button_selector))?;
                        match self.tab.wait_for_element(gig_review_el_selector.as_ref()) {
                            Ok(_) => continue,
                            Err(_) => break,
                        }
                    }
                    Err(_) => break,
                }
            };
            let country_selector = ".country p";
            let country = gig_review_el
                .find_element(country_selector)
                .map_err(Self::map_err_fn(
                    gig_review_el_selector.append(country_selector),
                ))?
                .get_inner_text()
                .map_err(Self::map_err_fn(
                    gig_review_el_selector.append(country_selector),
                ))?;

            let rating_selector = "strong.rating-score";
            let rating = gig_review_el
                .find_element(country_selector)
                .map_err(Self::map_err_fn(
                    gig_review_el_selector.append(rating_selector),
                ))?
                .get_inner_text()
                .map_err(Self::map_err_fn(
                    gig_review_el_selector.append(rating_selector),
                ))?;

            let show_more_button_selector = ".expand-button button";
            let show_more_button_el = gig_review_el.find_element(show_more_button_selector);
            if let Ok(button) = show_more_button_el {
                button.click().map_err(Self::map_err_fn(
                    gig_review_el_selector.append(show_more_button_selector),
                ))?;
                sleep(Duration::from_millis(500));
            }

            let description_selector = gig_review_el_selector.append(".review-description p");
            let description = gig_review_el
                .find_element(description_selector.as_ref())
                .map_err(Self::map_err_fn(description_selector.to_owned()))?
                .get_inner_text()
                .map_err(Self::map_err_fn(description_selector))?;

            let price_duration_els_selector = Selector::new(
                "div:has(p:nth-child(1)):has(p:nth-child(2):last-child) > p:first-child".to_owned(),
            );
            let mut price_duration_els = gig_review_el
                .find_elements(price_duration_els_selector.as_ref())
                .map_err(Self::map_err_fn(price_duration_els_selector.to_owned()))?
                .into_iter();
            let price = price_duration_els
                .next()
                .ok_or(GigReviewsError::ElementNotFound("price".to_owned()))?
                .get_inner_text()
                .map_err(Self::map_err_fn(price_duration_els_selector.nth_child(1)))?;
            let duration = price_duration_els
                .next()
                .ok_or(GigReviewsError::ElementNotFound("duration".to_owned()))?
                .get_inner_text()
                .map_err(Self::map_err_fn(price_duration_els_selector.nth_child(2)))?;
            gig_reviews.push(GigReview {
                country,
                rating,
                description,
                price,
                duration,
            })
        }

        Ok(gig_reviews)
    }
}
