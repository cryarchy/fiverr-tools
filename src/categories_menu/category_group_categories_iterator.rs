use std::sync::Arc;

use headless_chrome::Tab;

use crate::{selector::Selector, wrapped_element::WrappedElement};

use super::error::CategoriesMenuError;

#[derive(Debug)]
pub struct Category {
    pub name: String,
    pub url: String,
}

pub struct CategoryGroupCategoriesIterator<'a> {
    tab: &'a Arc<Tab>,
    current_index: usize,
    base_selector: Selector,
}

impl<'a> CategoryGroupCategoriesIterator<'a> {
    pub fn new(tab: &'a Arc<Tab>, base_selector: Selector) -> Self {
        Self {
            base_selector,
            tab,
            current_index: 2,
        }
    }

    fn get_category_el_selector(&self) -> Selector {
        self.base_selector
            .append(".sub-menu-item:not(.linked-title):not(.spotlight-item)")
            .nth_child(self.current_index)
            .append("a")
    }

    fn _next(&mut self) -> Result<Option<Category>, CategoriesMenuError> {
        let selector = self.get_category_el_selector();
        match self.tab.find_element(selector.as_ref()) {
            Ok(element) => {
                let wrapped_element = WrappedElement::new(element, selector.to_owned());
                let category_name = wrapped_element.get_inner_text()?;
                let href = wrapped_element.get_expected_attribute_value("href")?;

                self.current_index += 1;
                Ok(Some(Category {
                    name: category_name,
                    url: href,
                }))
            }
            Err(e) => {
                println!("{e} : {selector}");
                Ok(None)
            }
        }
    }
}

impl Iterator for CategoryGroupCategoriesIterator<'_> {
    type Item = Result<Category, CategoriesMenuError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(sub_category) => sub_category.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
