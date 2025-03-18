use crate::wrapped::WrappedTab;

use super::{
    category_group_categories_iterator::CategoryGroupCategoriesIterator,
    category_group_element::CategoryGroupElement, category_groups_iterator::CategoryGroupsIterator,
    main_categories_element::MainCategoryElement, main_categories_iterator::MainCategoriesIterator,
};

#[derive(Debug)]
pub struct Category {
    pub main_category: String,
    pub category_group: String,
    pub name: String,
    pub url: String,
}

pub struct CategoriesIterator<'a> {
    tab: &'a WrappedTab,
    main_categories_iterator: MainCategoriesIterator<'a>,
    current_main_category: MainCategoryElement<'a>,
    category_groups_iterator: CategoryGroupsIterator<'a>,
    current_category_group: CategoryGroupElement<'a>,
    category_group_categories_iterator: CategoryGroupCategoriesIterator<'a>,
}

impl<'a> CategoriesIterator<'a> {
    pub fn new(tab: &'a WrappedTab) -> Result<Self, crate::Error> {
        let mut main_categories_iterator = MainCategoriesIterator::new(tab);
        let current_main_category =
            main_categories_iterator
                .next()
                .ok_or(crate::Error::Unexpected(
                    "Empty main categories iterator".to_owned(),
                ))??;
        let mut category_groups_iterator =
            CategoryGroupsIterator::new(tab, current_main_category.selector().to_owned());
        let current_category_group =
            category_groups_iterator
                .next()
                .ok_or(crate::Error::Unexpected(
                    "Empty category group iterator".to_owned(),
                ))??;
        let category_group_categories_iterator =
            CategoryGroupCategoriesIterator::new(tab, current_category_group.selector().to_owned());
        Ok(Self {
            tab,
            main_categories_iterator,
            current_main_category,
            category_groups_iterator,
            current_category_group,
            category_group_categories_iterator,
        })
    }

    fn _next(&mut self) -> Result<Option<Category>, crate::Error> {
        match self.category_group_categories_iterator.next() {
            Some(category) => {
                let category = category?;
                Ok(Some(Category {
                    main_category: self.current_main_category.name()?,
                    category_group: self.current_category_group.name()?,
                    name: category.name,
                    url: category.url,
                }))
            }
            None => match self.category_groups_iterator.next() {
                Some(category_group) => {
                    self.current_category_group = category_group?;
                    self.category_group_categories_iterator = CategoryGroupCategoriesIterator::new(
                        self.tab,
                        self.current_category_group.selector().to_owned(),
                    );
                    self._next()
                }
                None => match self.main_categories_iterator.next() {
                    Some(main_category) => {
                        self.current_main_category = main_category?;
                        self.category_groups_iterator = CategoryGroupsIterator::new(
                            self.tab,
                            self.current_main_category.selector().to_owned(),
                        );
                        self._next()
                    }
                    None => Ok(None),
                },
            },
        }
    }
}

impl Iterator for CategoriesIterator<'_> {
    type Item = Result<Category, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(sub_category) => sub_category.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
