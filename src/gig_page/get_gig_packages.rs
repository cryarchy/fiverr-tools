use std::collections::HashMap;

use crate::{selector::Selector, string_cleaner::STRING_CLEANER};

use super::GigPage;

#[derive(Debug)]
pub struct GigPackage {
    pub r#type: String,
    pub price: usize,
    pub title: String,
    pub description: Option<String>,
    pub properties: HashMap<String, String>,
    pub delivery_time: Option<String>,
}

impl GigPage {
    pub fn get_gig_packages(&self) -> Result<Vec<GigPackage>, crate::Error> {
        let gig_package_header_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr.package-type th".to_owned());
        let gig_packages_header_els = self
            .tab
            .find_elements(&gig_package_header_els_selector, true)?;
        let mut gig_packages = Vec::new();
        for gig_package_header_el in gig_packages_header_els.iter().skip(1) {
            let price_el_selector = gig_package_header_el.selector().append(".price-wrapper");
            let price = self
                .tab
                .find_element(&price_el_selector)?
                .get_inner_text()?;
            let price = STRING_CLEANER.as_usize(&price)?;

            let type_el_selector = gig_package_header_el.selector().append(".type");
            let r#type = self.tab.find_element(&type_el_selector)?.get_inner_text()?;

            let title_el_selector = gig_package_header_el.selector().append(".title");
            let title = self
                .tab
                .find_element(&title_el_selector)?
                .get_inner_text()?;

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
            .find_elements(&gig_package_desc_els_selector, true)?;
        for (i, gig_packages_desc_el) in gig_packages_desc_els.iter().skip(1).enumerate() {
            let description = gig_packages_desc_el.get_inner_text()?;
            gig_packages[i].description = Some(description);
        }

        let check_svg_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr td span:has(svg)".to_owned());
        let check_svg_els = self.tab.find_elements(&check_svg_els_selector, false)?;
        let count_classes = |class: &str| class.split(' ').collect::<Vec<_>>().len();
        let mut checked_mark_class_count = 0;
        for svg_span_el in check_svg_els.into_iter() {
            if let Some(class) = svg_span_el.get_attribute_value("class")? {
                log::debug!("SVG span class: {class}");
                if count_classes(&class) > checked_mark_class_count {
                    checked_mark_class_count = count_classes(&class);
                }
            }
        }

        log::debug!("SVG span class count for checked: {checked_mark_class_count}");
        let table_rows_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr".to_owned());
        let table_rows = self.tab.find_elements(&table_rows_selector, true)?;
        for table_row in table_rows.into_iter() {
            if table_row.get_attribute_value("class")?.is_some() {
                continue;
            }
            let table_data_selector = table_row.selector().append("td");
            let row_data_els = self.tab.find_elements(&table_data_selector, true)?;
            let row_property = row_data_els[0].get_inner_text()?;
            for (i, data_el) in row_data_els.iter().skip(1).enumerate() {
                let span_selector = data_el.selector().append("span:has(svg)");
                let data_value = match self.tab.find_element(&span_selector) {
                    Ok(span) => {
                        let span_class = span.get_attribute_value("class")?;
                        match span_class {
                            Some(span_class) => {
                                (count_classes(&span_class) == checked_mark_class_count).to_string()
                            }

                            None => {
                                return Err(crate::Error::Unexpected(
                                    "Encountered a span element without a class attribute"
                                        .to_owned(),
                                ));
                            }
                        }
                    }
                    Err(_) => data_el.get_inner_text()?,
                };
                gig_packages[i]
                    .properties
                    .insert(row_property.to_owned(), data_value);
            }
        }

        let gig_package_delivery_time_els_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gig-page-packages-table table tbody tr.delivery-time td".to_owned());
        let gig_package_delivery_time_els = self
            .tab
            .find_elements(&gig_package_delivery_time_els_selector, true)?;
        for (i, gig_package_delivery_time_el) in
            gig_package_delivery_time_els.iter().skip(1).enumerate()
        {
            let delivery_time_el_selector = gig_package_delivery_time_el
                .selector()
                .append("span:not([class])");
            let delivery_time = match self.tab.find_element(&delivery_time_el_selector) {
                Ok(element) => element.get_inner_text()?,
                Err(_) => gig_package_delivery_time_el.get_inner_text()?,
            };
            gig_packages[i].delivery_time = Some(delivery_time);
        }

        Ok(gig_packages)
    }
}
