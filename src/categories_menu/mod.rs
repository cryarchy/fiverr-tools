mod categories_iterator;
mod category_group_categories_iterator;
mod category_group_element;
mod category_groups_iterator;
mod main_categories_element;
mod main_categories_iterator;

use categories_iterator::CategoriesIterator;

use crate::{
    selector::Selector,
    wrapped::{WrappedElement, WrappedTab},
};

pub struct CategoriesMenu {
    tab: WrappedTab,
}

#[derive(Debug)]
pub struct GigCategory {
    main_category: String,
    sub_category: String,
    name: String,
    link: String,
}

impl CategoriesMenu {
    pub fn new(tab: WrappedTab) -> Self {
        Self { tab }
    }

    fn main_categories_selector() -> Selector {
        Selector::new(r#"#CategoriesMenu .categories li[data-level="top"]"#.to_owned())
    }

    fn find_element(&self, selector: &Selector) -> Result<WrappedElement, crate::Error> {
        self.tab.find_element(&selector).map_err(|e| e.into())
    }

    fn find_elements(&self, selector: &Selector) -> Result<Vec<WrappedElement>, crate::Error> {
        self.tab.find_elements(&selector).map_err(|e| e.into())
    }

    pub fn get_gig_categories(&self) -> Result<CategoriesIterator, crate::Error> {
        CategoriesIterator::new(&self.tab)
    }
}
