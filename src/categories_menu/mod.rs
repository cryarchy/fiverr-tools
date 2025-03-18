mod categories_iterator;
mod category_group_categories_iterator;
mod category_group_element;
mod category_groups_iterator;
mod error;
mod main_categories_element;
mod main_categories_iterator;

use categories_iterator::CategoriesIterator;
use error::CategoriesMenuError;
use headless_chrome::Tab;
use std::sync::Arc;

use crate::{
    selector::Selector,
    wrapped_element::{WrappedElement, WrappedElementError},
};

pub struct CategoriesMenu {
    tab: Arc<Tab>,
}

#[derive(Debug)]
pub struct GigCategory {
    main_category: String,
    sub_category: String,
    name: String,
    link: String,
}

impl CategoriesMenu {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    fn main_categories_selector() -> Selector {
        Selector::new(r#"#CategoriesMenu .categories li[data-level="top"]"#.to_owned())
    }

    fn find_element(&self, selector: &Selector) -> Result<WrappedElement, CategoriesMenuError> {
        self.tab
            .find_element(selector.as_ref())
            .map(|element| WrappedElement::new(element, selector.to_owned()))
            .map_err(|e| WrappedElementError::markup_interaction(e, selector).into())
    }

    fn find_elements(
        &self,
        selector: &Selector,
    ) -> Result<Vec<WrappedElement>, CategoriesMenuError> {
        self.tab
            .find_elements(selector.as_ref())
            .map(|elements| {
                elements
                    .into_iter()
                    .enumerate()
                    .map(|(i, element)| WrappedElement::new(element, selector.nth_child(i + 1)))
                    .collect()
            })
            .map_err(|e| WrappedElementError::markup_interaction(e, selector).into())
    }

    pub fn get_gig_categories(&self) -> Result<CategoriesIterator, CategoriesMenuError> {
        CategoriesIterator::new(&self.tab)
    }

    // pub fn get_gig_categories(&self) -> Result<Vec<Category>, CategoriesMenuError> {
    //     let selector = Self::main_categories_selector();
    //     let main_category_elements = self.find_elements(&selector)?;
    //     let mut categories = Vec::new();
    //     for main_category_element in main_category_elements.into_iter() {
    //         let main_category_element_selector = main_category_element.selector().append("a");
    //         let category_name = self
    //             .find_element(&main_category_element_selector)?
    //             .get_inner_text()?;
    //         println!("Category name: {category_name}");
    //         main_category_element.move_mouse_over()?;
    //         let menu_panel_selector = main_category_element.selector().append(".menu-panel");
    //         self.tab
    //             .wait_for_element(menu_panel_selector.as_ref())
    //             .map_err(|e| WrappedElementError::markup_interaction(e, &menu_panel_selector))?;
    //         let sub_categories_selector = menu_panel_selector.append(".menu-bucket");
    //         let sub_category_els = self.find_elements(&sub_categories_selector)?;
    //         let mut sub_categories = HashMap::new();
    //         for sub_category_el in sub_category_els.into_iter() {
    //             let sub_category_title_selector = sub_category_el
    //                 .selector()
    //                 .append(".linked-title:first-child :first-child");
    //             let sub_category_title = self
    //                 .find_element(&sub_category_title_selector)?
    //                 .get_inner_text()?;
    //             println!("Subcategory title: {sub_category_title}");

    //             let category_links_selector = sub_category_el
    //                 .selector()
    //                 .append(".sub-menu-item:not(.linked-title):not(.spotlight-item) a");
    //             let category_link_els = self.find_elements(&category_links_selector)?;
    //             let mut category_links = Vec::new();
    //             for category_link_el in category_link_els.into_iter() {
    //                 let text = category_link_el.get_inner_text()?.replace("NEW", "");
    //                 let target = category_link_el.get_expected_attribute_value("href")?;
    //                 category_links.push(CategoryLink { text, target });
    //             }
    //             sub_categories.insert(sub_category_title, category_links);
    //         }
    //         categories.push(Category {
    //             name: category_name,
    //             sub_categories,
    //         });
    //         println!();
    //     }

    //     Ok(categories)
    // }
}
