use std::sync::Arc;

use headless_chrome::Tab;

use crate::selector::Selector;

use super::{GigDataExtractor, error::MarkupInteractionError};

#[derive(Debug, thiserror::Error)]
pub enum GigFaqError {
    #[error("GigFaqError: {0}")]
    MarkupInteraction(#[from] MarkupInteractionError),
}

impl GigFaqError {
    fn markup_interaction(e: anyhow::Error, selector: String) -> Self {
        Self::MarkupInteraction(MarkupInteractionError::new(e, selector))
    }
}

#[derive(Debug)]
pub struct GigFaq {
    question: String,
    answer: String,
}

pub struct GigFaqIterator {
    tab: Arc<Tab>,
    current_index: usize,
}

impl GigFaqIterator {
    fn new(tab: Arc<Tab>) -> Result<Self, GigFaqError> {
        Ok(Self {
            current_index: 1,
            tab,
        })
    }
}

impl Iterator for GigFaqIterator {
    type Item = Result<GigFaq, GigFaqError>;

    fn next(&mut self) -> Option<Self::Item> {
        fn _next(mut_self: &mut GigFaqIterator) -> Result<Option<GigFaq>, GigFaqError> {
            let gig_faq_els_selector = Selector::new(
                "#main-wrapper > .main-content .gig-page > .main article.faq-collapsible"
                    .to_owned(),
            )
            .nth_child(mut_self.current_index);
            mut_self.current_index += 1;
            if mut_self
                .tab
                .find_element(gig_faq_els_selector.as_ref())
                .is_err()
            {
                return Ok(None);
            }

            let question_selector = gig_faq_els_selector.append(".faq-collapsible-title p");
            let question = mut_self
                .tab
                .find_element(question_selector.as_ref())
                .map_err(|e| GigFaqError::markup_interaction(e, question_selector.to_string()))?
                .get_inner_text()
                .map_err(|e| GigFaqError::markup_interaction(e, question_selector.to_string()))?;

            let answer_selector = gig_faq_els_selector.append(".faq-collapsible-content p");
            let answer = mut_self
                .tab
                .find_element(answer_selector.as_ref())
                .map_err(|e| GigFaqError::markup_interaction(e, answer_selector.to_string()))?
                .get_inner_text()
                .map_err(|e| GigFaqError::markup_interaction(e, answer_selector.to_string()))?;

            Ok(Some(GigFaq { question, answer }))
        }
        match _next(self) {
            Ok(gig_faq) => gig_faq.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}

impl GigDataExtractor {
    pub fn extract_gig_faqs(
        &self,
    ) -> Result<impl IntoIterator<Item = Result<GigFaq, GigFaqError>>, GigFaqError> {
        GigFaqIterator::new(self.tab.clone())
    }

    pub fn _extract_gig_faqs(&self) -> Result<Vec<GigFaq>, GigFaqError> {
        let gig_faq_els_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main article.faq-collapsible".to_owned(),
        );
        let gig_faq_els = self
            .tab
            .find_elements(gig_faq_els_selector.as_ref())
            .map_err(Self::map_err_fn(gig_faq_els_selector.to_owned()))?;

        let mut gig_faqs = Vec::new();
        for (i, gig_faq_el) in gig_faq_els.into_iter().enumerate() {
            let get_child_selector =
                |child_selector: &str| gig_faq_els_selector.nth_child(i).append(child_selector);

            let question_selector = ".faq-collapsible-title p";
            let question = gig_faq_el
                .find_element(question_selector)
                .map_err(Self::map_err_fn(get_child_selector(question_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(question_selector)))?;

            let answer_selector = ".faq-collapsible-content p";
            let answer = gig_faq_el
                .find_element(answer_selector)
                .map_err(Self::map_err_fn(get_child_selector(answer_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(answer_selector)))?;

            gig_faqs.push(GigFaq { question, answer });
        }

        Ok(gig_faqs)
    }
}
