// Start Google Chrome with:
//     $ google-chrome --remote-debugging-port=9222 --user-data-dir="/tmp/chrome_dev"
// and update the websocket URL below

mod app_config;
mod categories_menu;
mod db;
mod error;
mod gig_page;
mod gigs_page;
mod markup_interaction_error;
mod selector;
mod site_nav;
mod string_cleaner;
mod wrapped;

use std::{sync::LazyLock, thread::sleep, time::Duration};

use anyhow::Result;
use app_config::AppConfig;
use categories_menu::CategoriesMenu;
use db::{
    gig_category_gigs::GigCategoryGigs,
    gig_category_repo::{GigCategoryRepo, ScrapeGigsOutput},
    gig_repo::GigRepo,
};
use error::Error;
use figment::{
    Figment,
    providers::{Format, Yaml},
};
use flexi_logger::Logger;
use gig_page::GigPage;
use gigs_page::GigsPage;
use headless_chrome::Browser;
use site_nav::SiteNav;
use sqlx::postgres::PgPoolOptions;
use url::Url;
use wrapped::WrappedTab;

static BASE_URL: LazyLock<Url> = LazyLock::new(|| Url::parse("https://www.fiverr.com/").unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    let app_config: AppConfig = Figment::new()
        .merge(Yaml::file("app-config.yaml"))
        .extract()?;

    Logger::try_with_str(app_config.log_level)?.start()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;

    let gig_category_repo = GigCategoryRepo::new(pool.clone());
    let gig_repo = GigRepo::new(pool.clone());
    let gig_category_gigs = GigCategoryGigs::new(pool.clone());

    let browser = Browser::connect(app_config.browser_ws_url)?;

    let tab = browser.new_tab()?;
    tab.close(false)?;

    // Get the list of existing tabs/pages
    let fiverr_tab = {
        let tabs = browser.get_tabs().lock().unwrap();

        let mut fiverr_tab = None;

        for tab in tabs.iter() {
            if tab.get_url().contains("fiverr") {
                fiverr_tab = Some(tab);
                break;
            }
        }

        WrappedTab::new((*fiverr_tab.unwrap()).clone())
    };

    log::info!("Fiverr tab title: {}", fiverr_tab.get_title()?);

    loop {
        let page_nav = SiteNav::new(fiverr_tab.clone());
        page_nav.go_home()?;

        let categories_menu = CategoriesMenu::new(fiverr_tab.clone());
        // get a gig category
        for category in categories_menu.get_gig_categories()? {
            let category = category?;
            let category_url = BASE_URL.join(&category.url)?;
            log::info!(
                "Processing: {} > {} > {} ({})",
                category.main_category,
                category.category_group,
                category.name,
                category_url
            );

            // get the record associated with the gig category
            let category_record = gig_category_repo
                .get_scrape_gigs(category_url.path())
                .await?;
            let category_record = match category_record {
                Some(category_record) => category_record,
                None => {
                    // create a record for the gig category if one does not exist
                    let category_name = category.name.to_owned();
                    let category_record_id = gig_category_repo
                        .create(&db::gig_category_repo::CreateParams {
                            path: category_url.path().to_owned(),
                            name: category.name,
                            sub_group_name: category.category_group,
                            main_group_name: category.main_category,
                        })
                        .await?;
                    ScrapeGigsOutput {
                        scrape_gigs: false,
                        name: category_name,
                        id: category_record_id,
                    }
                }
            };

            // skip scraping the record if scrape_gigs = false
            if !category_record.scrape_gigs {
                continue;
            }

            // delete all partially scraped gigs
            let deleted_records_count = gig_repo.delete_partially_scraped_gigs().await?;
            log::info!("Deleted {deleted_records_count} partially scraped gigs");

            // get the minimum count of all gigs per category
            let minimum_gigs_per_category =
                gig_category_gigs.least_gigs_count_for_categories().await?;

            // get the gig count for the current category
            let current_gig_category_gigs_count =
                gig_repo.count_for_category(category_record.id).await?;

            // skip scraping if the current category does not have a gigs count equal to the lowest gigs count per category
            if current_gig_category_gigs_count != minimum_gigs_per_category {
                continue;
            }

            // navigate to the current category's gigs page
            fiverr_tab.navigate_to(category_url.as_str())?;
            fiverr_tab.wait_for_element_with_custom_timeout(
                &GigsPage::gig_els_selector(),
                Duration::from_secs(60),
            )?;

            // get the page of the last scraped gig
            let mut last_gigs_page = match minimum_gigs_per_category {
                0 => 1,
                _ => match gig_repo
                    .get_page_of_last_scraped_gig(category_record.id)
                    .await?
                {
                    Some(page) => page,
                    None => {
                        log::warn!("Application not expected to reach this point!");
                        1
                    }
                },
            };

            // find the first gig within the category that does not have an associated record in the database
            let gigs_page = GigsPage::new(fiverr_tab.clone());
            struct GigToScrape {
                url: Url,
                page: usize,
            }
            let mut gig_to_scrape = None;
            loop {
                // go the the gigs page from which the last scraped gig was found
                log::info!("Navigating to page {last_gigs_page}");
                if !gigs_page.go_to_page(last_gigs_page as usize)? {
                    break;
                }

                for gig_card in gigs_page.gigs()? {
                    let gig_card = gig_card?;
                    let gig_url = BASE_URL.join(&gig_card.url)?;
                    let gig_path = gig_url.path();
                    let gig_record_exists = gig_repo.exists_by_path(gig_path).await?;
                    if gig_record_exists {
                        log::info!("{gig_path} - scraped");
                        continue;
                    }
                    gig_to_scrape = Some(GigToScrape {
                        url: gig_url,
                        page: gig_card.page,
                    });
                    break;
                }

                last_gigs_page += 1;
            }

            let Some(gig_to_scrape) = gig_to_scrape else {
                log::warn!(
                    "No more gigs to scrape in the {} category!",
                    category_record.name
                );
                continue;
            };

            // navigate to the gig's page
            fiverr_tab.navigate_to(gig_to_scrape.url.as_str())?;
            fiverr_tab.wait_for_element_with_custom_timeout(
                &GigPage::title_selector(),
                Duration::from_secs(60),
            )?;

            let gig_page = GigPage::new(fiverr_tab.clone());

            log::info!("Gig title: {}", gig_page.get_title()?);
            log::info!("Rating: {}", gig_page.get_gig_rating()?);
            log::info!("Reviews count: {}", gig_page.get_gig_reviews_count()?);
            log::info!("Description: {}", gig_page.get_gig_description()?);
            log::info!("Metadata:");
            let gig_metadata = gig_page.get_gig_metadata()?;
            for entry in gig_metadata {
                log::info!("\t {:?}", entry);
            }
            log::info!("Seller rating: {}", gig_page.get_seller_rating()?);
            log::info!(
                "Seller ratings count: {}",
                gig_page.get_seller_ratings_count()?
            );
            log::info!("Seller level: {}", gig_page.get_seller_level()?);
            log::info!("Seller stats:");
            let seller_stats = gig_page.get_seller_stats()?;
            for entry in seller_stats {
                log::info!("\t {:?}", entry);
            }
            log::info!("Seller description: {}", gig_page.get_seller_description()?);
            let gallery_visuals = gig_page.get_gig_visuals()?;
            log::info!("Gallery visuals:");
            for entry in gallery_visuals {
                log::info!("\t {:?}", entry);
            }
            log::info!("Gig packages:");
            let gig_packages = gig_page.get_gig_packages()?;
            log::info!("{:#?}", gig_packages);
            log::info!("Gig FAQs:");
            for result in gig_page.get_gig_faqs()? {
                log::info!("{:#?}", result?);
            }
            log::info!("Gig reviews:");
            for gig_review_result in gig_page.get_gig_reviews()? {
                log::info!("{:#?}", gig_review_result?);
                break;
            }
        }
    }

    Ok(())
}
