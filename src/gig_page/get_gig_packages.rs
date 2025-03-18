use std::collections::HashMap;

use crate::{
    markup_interaction_error::MarkupInteractionError,
    selector::Selector,
    string_cleaner::{STRING_CLEANER, StringCleanerError},
};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum GigPackagesError {
    #[error("GigPackagesError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
    #[error("GigPackagesError: {0}")]
    StringCleanerError(#[from] StringCleanerError),
    #[error("GigPackagesError: Unexpected '{0}'")]
    Unexpected(String),
}

#[derive(Debug)]
pub struct GigPackage {
    r#type: String,
    price: usize,
    title: String,
    description: Option<String>,
    properties: HashMap<String, String>,
    delivery_time: Option<String>,
}

impl GigPage {
    pub fn get_gig_packages(&self) -> Result<Vec<GigPackage>, GigPackagesError> {
        let gig_package_header_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr.package-type th".to_owned());
        let gig_packages_header_els = self
            .tab
            .find_elements(gig_package_header_els_selector.as_ref())
            .map_err(Self::map_err_fn(gig_package_header_els_selector.to_owned()))?;
        let mut gig_packages = Vec::new();
        for (i, gig_package_header_el) in gig_packages_header_els.iter().skip(1).enumerate() {
            let get_child_selector = |child_selector: &str| {
                gig_package_header_els_selector
                    .nth_child(i)
                    .append(child_selector)
            };

            let price_el_selector = ".price-wrapper";
            let price = gig_package_header_el
                .find_element(price_el_selector)
                .map_err(Self::map_err_fn(get_child_selector(price_el_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(price_el_selector)))?;
            let price = STRING_CLEANER.as_usize(&price)?;

            let type_el_selector = ".type";
            let r#type = gig_package_header_el
                .find_element(type_el_selector)
                .map_err(Self::map_err_fn(get_child_selector(type_el_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(type_el_selector)))?;

            let title_el_selector = ".title";
            let title = gig_package_header_el
                .find_element(title_el_selector)
                .map_err(Self::map_err_fn(get_child_selector(title_el_selector)))?
                .get_inner_text()
                .map_err(Self::map_err_fn(get_child_selector(title_el_selector)))?;

            gig_packages.push(GigPackage {
                r#type,
                price,
                title,
                description: None,
                delivery_time: None,
                properties: HashMap::new(),
            })
        }

        let gig_package_desc_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr.description td".to_owned());
        let gig_packages_desc_els = self
            .tab
            .find_elements(gig_package_desc_els_selector.as_ref())
            .map_err(Self::map_err_fn(gig_package_desc_els_selector.to_owned()))?;
        for (i, gig_packages_desc_el) in gig_packages_desc_els.iter().skip(1).enumerate() {
            let description = gig_packages_desc_el
                .get_inner_text()
                .map_err(Self::map_err_fn(gig_package_desc_els_selector.nth_child(i)))?;
            gig_packages[i].description = Some(description);
        }

        let check_svg_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr td span:has(svg)".to_owned());
        let check_svg_els = self
            .tab
            .find_elements(check_svg_els_selector.as_ref())
            .map_err(Self::map_err_fn(check_svg_els_selector.to_owned()))?;
        let count_classes = |class: &str| class.split(' ').collect::<Vec<_>>().len();
        let mut checked_mark_class_count = 0;
        for (i, svg_span_el) in check_svg_els.into_iter().enumerate() {
            if let Some(class) = svg_span_el
                .get_attribute_value("class")
                .map_err(Self::map_err_fn(check_svg_els_selector.nth_child(i)))?
            {
                println!("SVG span class: {class}");
                if count_classes(&class) > checked_mark_class_count {
                    checked_mark_class_count = count_classes(&class);
                }
            }
        }

        println!("SVG span class count for checked: {checked_mark_class_count}");
        let table_rows_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr:not([class])".to_owned());
        let table_rows = self
            .tab
            .find_elements(table_rows_selector.as_ref())
            .map_err(Self::map_err_fn(table_rows_selector.to_owned()))?;
        for (i, table_row) in table_rows.into_iter().enumerate() {
            let table_data_selector = "td";
            let absolute_data_el_selector =
                || table_rows_selector.nth_child(i).append(table_data_selector);
            let row_data_els = table_row
                .find_elements(table_data_selector)
                .map_err(Self::map_err_fn(absolute_data_el_selector()))?;
            let row_property = row_data_els[0]
                .get_inner_text()
                .map_err(Self::map_err_fn(absolute_data_el_selector().nth_child(0)))?;
            for (i, data_el) in row_data_els.iter().skip(1).enumerate() {
                let span_selector = "span:has(svg)";
                let data_value = match data_el.find_element(span_selector) {
                    Ok(span) => {
                        let span_class =
                            span.get_attribute_value("class").map_err(Self::map_err_fn(
                                absolute_data_el_selector()
                                    .append(span_selector)
                                    .nth_child(i),
                            ))?;
                        match span_class {
                            Some(span_class) => {
                                (count_classes(&span_class) == checked_mark_class_count).to_string()
                            }

                            None => {
                                return Err(GigPackagesError::Unexpected(
                                    "Encountered a span element without a class attribute"
                                        .to_owned(),
                                ));
                            }
                        }
                    }
                    Err(_) => data_el
                        .get_inner_text()
                        .map_err(Self::map_err_fn(absolute_data_el_selector().nth_child(i)))?,
                };
                gig_packages[i]
                    .properties
                    .insert(row_property.to_owned(), data_value);
            }
        }

        let gig_package_delivery_time_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr.delivery-time td".to_owned());
        let gig_package_delivery_time_els = self
            .tab
            .find_elements(gig_package_delivery_time_els_selector.as_ref())
            .map_err(Self::map_err_fn(
                gig_package_delivery_time_els_selector.to_owned(),
            ))?;
        for (i, gig_package_delivery_time_el) in
            gig_package_delivery_time_els.iter().skip(1).enumerate()
        {
            let delivery_time_el_selector = "span:not([class])";
            let delivery_time = gig_package_delivery_time_el
                .find_element(delivery_time_el_selector)
                .map_err(Self::map_err_fn(
                    gig_package_delivery_time_els_selector
                        .nth_child(i)
                        .append(delivery_time_el_selector),
                ))?
                .get_inner_text()
                .map_err(Self::map_err_fn(
                    gig_package_delivery_time_els_selector
                        .nth_child(i)
                        .append(delivery_time_el_selector),
                ))?;
            gig_packages[i].delivery_time = Some(delivery_time);
        }

        Ok(gig_packages)
    }
}
