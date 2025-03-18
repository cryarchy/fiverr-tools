use std::sync::Arc;

use headless_chrome::Tab;

use crate::{selector::Selector, wrapped_element::WrappedElement};

use super::{category_group_element::CategoryGroupElement, error::CategoriesMenuError};

pub struct CategoryGroupsIterator<'a> {
    tab: &'a Arc<Tab>,
    current_index: usize,
    base_selector: Selector,
}

impl<'a> CategoryGroupsIterator<'a> {
    pub fn new(tab: &'a Arc<Tab>, base_selector: Selector) -> Self {
        Self {
            base_selector,
            tab,
            current_index: 1,
        }
    }

    pub fn selector(&self) -> Selector {
        self.base_selector
            .append(".menu-bucket")
            .nth_child(self.current_index)
    }

    fn _next(&mut self) -> Result<Option<CategoryGroupElement<'a>>, CategoriesMenuError> {
        let selector = self.selector();
        match self.tab.find_element(selector.as_ref()) {
            Ok(element) => {
                let wrapped_element = WrappedElement::new(element, selector.to_owned());

                self.current_index += 1;
                Ok(Some(CategoryGroupElement::new(self.tab, wrapped_element)))
            }
            Err(e) => {
                println!("{e} : {selector}");
                Ok(None)
            }
        }
    }
}

impl<'a> Iterator for CategoryGroupsIterator<'a> {
    type Item = Result<CategoryGroupElement<'a>, CategoriesMenuError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(sub_category) => sub_category.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
