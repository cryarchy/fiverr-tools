use std::sync::Arc;

use headless_chrome::Tab;

use crate::{
    selector::Selector,
    wrapped_element::{WrappedElement, WrappedElementError},
};

use super::{error::CategoriesMenuError, main_categories_element::MainCategoryElement};

pub struct MainCategoriesIterator<'a> {
    tab: &'a Arc<Tab>,
    current_index: usize,
}

impl<'a> MainCategoriesIterator<'a> {
    pub fn new(tab: &'a Arc<Tab>) -> Self {
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

    fn _next(&mut self) -> Result<Option<MainCategoryElement<'a>>, CategoriesMenuError> {
        let selector = Self::selector().nth_child(self.current_index);
        match self.tab.find_element(selector.as_ref()) {
            Ok(element) => {
                let wrapped_element = WrappedElement::new(element, selector.to_owned());
                wrapped_element.move_mouse_over()?;
                let main_category_element = MainCategoryElement::new(self.tab, wrapped_element);
                let panel_selector = main_category_element.menu_panel_selector();
                if self.tab.wait_for_element(panel_selector.as_ref()).is_err() {
                    let right_nav_btn_selector = Self::right_nav_btn_selector();
                    self.tab
                        .find_element(right_nav_btn_selector.as_ref())
                        .map_err(|e| {
                            WrappedElementError::markup_interaction(e, &right_nav_btn_selector)
                        })?;
                    self.tab
                        .wait_for_element(panel_selector.as_ref())
                        .map_err(|e| WrappedElementError::markup_interaction(e, &panel_selector))?;
                }

                self.current_index += 1;
                Ok(Some(main_category_element))
            }
            Err(e) => {
                println!("{e} : {selector}");
                Ok(None)
            }
        }
    }
}

impl<'a> Iterator for MainCategoriesIterator<'a> {
    type Item = Result<MainCategoryElement<'a>, CategoriesMenuError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok(main_category_element) => main_category_element.map(Ok),
            Err(e) => Some(Err(e)),
        }
    }
}
