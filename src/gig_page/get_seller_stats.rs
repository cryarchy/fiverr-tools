use crate::{selector::Selector, string_cleaner::STRING_CLEANER};

use super::GigPage;

#[derive(Debug)]
pub struct SellerStat {
    pub name: String,
    pub value: String,
}

impl GigPage {
    pub fn get_seller_stats(&self) -> Result<Vec<SellerStat>, crate::Error> {
        let seller_stats_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .seller-card .user-stats > li"
                .to_owned(),
        );
        let seller_stat_els = self.tab.find_elements(&seller_stats_selector)?;
        let mut seller_stats = Vec::new();

        for seller_stat_el in seller_stat_els.into_iter() {
            let name = seller_stat_el.get_inner_text()?;
            let value_selector = seller_stat_el.selector().append("strong");
            let value = self.tab.find_element(&value_selector)?.get_inner_text()?;
            let name = STRING_CLEANER.as_simple_text(&name)?;
            let value = STRING_CLEANER.as_simple_text(&value)?;
            seller_stats.push(SellerStat { name, value });
        }
        Ok(seller_stats)
    }
}
