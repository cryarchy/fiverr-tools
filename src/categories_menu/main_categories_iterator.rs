use crate::{selector::Selector, wrapped::WrappedTab};

use super::main_categories_element::MainCategoryElement;

pub struct MainCategoriesIterator<'a> {
    tab: &'a WrappedTab,
    current_index: usize,
}

impl<'a> MainCategoriesIterator<'a> {
    pub fn new(tab: &'a WrappedTab) -> Self {
        Self {
            tab,
            current_index: 1,
        }
    }

    pub fn selector() -> Selector {
        Selector::new(r#"#CategoriesMenu .categories li[data-level="top"]"#.to_owned())
    }

    fn right_nav_btn_selector() -> Selector {
        Selector::new(r#"#CategoriesMenu nav .right"#.to_owned())
    }

    fn _next(&mut self) -> Result<Option<MainCategoryElement<'a>>, crate::Error> {
        let selector = Self::selector().nth_child(self.current_index);
        match self.tab.find_element(&selector) {
            Ok(element) => {
                element.move_mouse_over()?;
                let main_category_element = MainCategoryElement::new(self.tab, element);
                let panel_selector = main_category_element.menu_panel_selector();
                if self.tab.wait_for_element(&panel_selector).is_err() {
                    let right_nav_btn_selector = Self::right_nav_btn_selector();
                    self.tab.find_element(&right_nav_btn_selector)?;
                    self.tab.wait_for_element(&panel_selector)?;
                }

                self.current_index += 1;
                Ok(Some(main_category_element))
            }
            Err(e) => {
                log::debug!("{e} : {selector}");
                Ok(None)
            }
        }
    }
}

impl<'a> Iterator for MainCategoriesIterator<'a> {
    type Item = Result<MainCategoryElement<'a>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(main_category_element) => main_category_element.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
