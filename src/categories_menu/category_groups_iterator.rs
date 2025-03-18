use crate::{selector::Selector, wrapped::WrappedTab};

use super::category_group_element::CategoryGroupElement;

pub struct CategoryGroupsIterator<'a> {
    tab: &'a WrappedTab,
    current_index: usize,
    base_selector: Selector,
}

impl<'a> CategoryGroupsIterator<'a> {
    pub fn new(tab: &'a WrappedTab, base_selector: Selector) -> Self {
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

    fn _next(&mut self) -> Result<Option<CategoryGroupElement<'a>>, crate::Error> {
        let selector = self.selector();
        match self.tab.find_element(&selector) {
            Ok(element) => {
                self.current_index += 1;
                Ok(Some(CategoryGroupElement::new(self.tab, element)))
            }
            Err(e) => {
                println!("{e} : {selector}");
                Ok(None)
            }
        }
    }
}

impl<'a> Iterator for CategoryGroupsIterator<'a> {
    type Item = Result<CategoryGroupElement<'a>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(sub_category) => sub_category.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
