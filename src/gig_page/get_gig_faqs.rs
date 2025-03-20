use crate::{selector::Selector, wrapped::WrappedTab};

use super::GigPage;

#[derive(Debug)]
pub struct GigFaq {
    pub question: String,
    pub answer: String,
}

pub struct GigFaqIterator {
    tab: WrappedTab,
    current_index: usize,
}

impl GigFaqIterator {
    fn new(tab: WrappedTab) -> Result<Self, crate::Error> {
        Ok(Self {
            current_index: 1,
            tab,
        })
    }
}

impl Iterator for GigFaqIterator {
    type Item = Result<GigFaq, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        fn _next(mut_self: &mut GigFaqIterator) -> Result<Option<GigFaq>, crate::Error> {
            let gig_faq_els_selector = Selector::new(
                "#main-wrapper > .main-content .gig-page > .main article.faq-collapsible"
                    .to_owned(),
            )
            .nth_child(mut_self.current_index);

            if mut_self.tab.find_element(&gig_faq_els_selector).is_err() {
                return Ok(None);
            }

            let question_selector = gig_faq_els_selector.append(".faq-collapsible-title p");
            let question = mut_self
                .tab
                .find_element(&question_selector)?
                .get_inner_text()?;

            let answer_selector = gig_faq_els_selector.append(".faq-collapsible-content p");
            let answer = mut_self
                .tab
                .find_element(&answer_selector)?
                .get_inner_text()?;

            mut_self.current_index += 1;
            Ok(Some(GigFaq { question, answer }))
        }
        match _next(self) {
            Ok(gig_faq) => gig_faq.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}

impl GigPage {
    pub fn get_gig_faqs(
        &self,
    ) -> Result<impl IntoIterator<Item = Result<GigFaq, crate::Error>>, crate::Error> {
        GigFaqIterator::new(self.tab.clone())
    }
}
