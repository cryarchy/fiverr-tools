mod categories_iterator;
mod category_group_categories_iterator;
mod category_group_element;
mod category_groups_iterator;
mod main_categories_element;
mod main_categories_iterator;

use categories_iterator::CategoriesIterator;

use crate::wrapped::WrappedTab;

pub struct CategoriesMenu {
    tab: WrappedTab,
}

impl CategoriesMenu {
    pub fn new(tab: WrappedTab) -> Self {
        Self { tab }
    }

    pub fn get_gig_categories(&self) -> Result<CategoriesIterator, crate::Error> {
        CategoriesIterator::new(&self.tab)
    }
}
