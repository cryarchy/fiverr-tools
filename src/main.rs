mod app_config;
mod categories_menu;
mod db;
mod error;
mod gig_page;
mod gigs_page;
mod markup_interaction_error;
mod price_range_parser;
mod selector;
mod site_nav;
mod string_cleaner;
mod wrapped;

use std::{sync::LazyLock, time::Duration};

use anyhow::Result;
use app_config::AppConfig;
use categories_menu::CategoriesMenu;
use db::{
    gig_category_gigs::GigCategoryGigs,
    gig_category_repo::{GigCategoryRepo, ScrapeGigsOutput},
    gig_faq_repo::GigFaqRepo,
    gig_metadata_repo::GigMetadataRepo,
    gig_package_feature_repo::GigPackageFeatureRepo,
    gig_package_repo::GigPackageRepo,
    gig_package_type_lookup::GigPackageTypeLookup,
    gig_repo::GigRepo,
    gig_review_repo::GigReviewRepo,
    seller_repo::SellerRepo,
    seller_stat_repo::SellerStatRepo,
    visual_repo::VisualRepo,
    visual_type_lookup::VisualTypeLookup,
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
use price_range_parser::PRICE_RANGE_PARSER;
use site_nav::SiteNav;
use sqlx::postgres::PgPoolOptions;
use string_cleaner::STRING_CLEANER;
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
    let seller_repo = SellerRepo::new(pool.clone());
    let seller_stat_repo = SellerStatRepo::new(pool.clone());
    let gig_metadata_repo = GigMetadataRepo::new(pool.clone());
    let visual_type_lookup = VisualTypeLookup::new(pool.clone());
    let visual_repo = VisualRepo::new(pool.clone());
    let gig_package_type_lookup = GigPackageTypeLookup::new(pool.clone());
    let gig_package_repo = GigPackageRepo::new(pool.clone());
    let gig_package_feature_repo = GigPackageFeatureRepo::new(pool.clone());
    let gig_faq_repo = GigFaqRepo::new(pool.clone());
    let gig_review_repo = GigReviewRepo::new(pool.clone());

    let browser =
        Browser::connect_with_timeout(app_config.browser_ws_url, Duration::from_secs(600))?;

    let browser_clone = browser.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let tabs = browser_clone.get_tabs().lock().unwrap();

            let mut fiverr_tab = None;

            for tab in tabs.iter() {
                if tab.get_url().contains("fiverr") {
                    fiverr_tab = Some(tab);
                    break;
                }
            }

            (*fiverr_tab.unwrap()).evaluate("void 0", false).unwrap();
        }
    });

    let tab = browser.new_tab()?;
    tab.close(false)?;

    // Get the list of existing tabs/pages
    let get_fiverr_tab = || {
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

    let fiverr_tab = get_fiverr_tab();
    log::info!("Fiverr tab title: {}", fiverr_tab.get_title()?);

    loop {
        let page_nav = SiteNav::new(get_fiverr_tab());
        page_nav.go_home()?;

        let categories_menu = CategoriesMenu::new(get_fiverr_tab());
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
            let fiverr_tab = get_fiverr_tab();
            fiverr_tab.navigate_to(category_url.as_str())?;
            fiverr_tab.wait_for_element_with_custom_timeout(
                &GigsPage::page_number_els_selector(),
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
            let gigs_page = GigsPage::new(get_fiverr_tab());
            let mut gig_to_scrape = None;
            'outer: loop {
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
                    gig_to_scrape = Some(gig_url);
                    break 'outer;
                }

                last_gigs_page += 1;
            }

            let Some(gig_to_scrape_url) = gig_to_scrape else {
                log::warn!(
                    "No more gigs to scrape in the {} category!",
                    category_record.name
                );
                continue;
            };

            // navigate to the gig's page
            let fiverr_tab = get_fiverr_tab();
            fiverr_tab.navigate_to(gig_to_scrape_url.as_str())?;
            fiverr_tab.wait_for_element_with_custom_timeout(
                &GigPage::seller_stats_selector(),
                Duration::from_secs(60),
            )?;
            fiverr_tab.wait_for_element_with_custom_timeout(
                &GigPage::title_selector(),
                Duration::from_secs(60),
            )?;

            let gig_page = GigPage::new(get_fiverr_tab());

            let Some(seller_username) = gig_to_scrape_url.path().split("/").nth(1) else {
                log::error!("Error getting seller's username from gig page's URL!");
                break;
            };

            let seller_id = match seller_repo.get_id(seller_username).await? {
                Some(seller_id) => seller_id,
                None => {
                    // scrape seller information if the seller's information does not exist in the database
                    let seller_rating = gig_page.get_seller_rating()?;
                    log::info!("Seller rating: {seller_rating}");
                    let seller_level = gig_page.get_seller_level()?;
                    log::info!("Seller level: {seller_level}");
                    let seller_review_count = gig_page.get_seller_ratings_count()?;
                    log::info!("Seller reviews count: {seller_review_count}");
                    let seller_description = gig_page.get_seller_description()?;
                    log::info!("Seller description: {seller_description}");
                    let create_params = db::seller_repo::CreateParams {
                        username: seller_username.to_owned(),
                        rating: seller_rating,
                        level: seller_level,
                        reviews_count: seller_review_count as i64,
                        description: seller_description,
                    };
                    let seller_id = seller_repo.create(&create_params).await?;

                    log::info!("Seller stats:");
                    let seller_stats = gig_page.get_seller_stats()?;
                    for stat in seller_stats {
                        log::info!("\t {}: {}", stat.name, stat.value);
                        let create_params = db::seller_stat_repo::CreateParams {
                            seller_id,
                            key: stat.name,
                            value: stat.value,
                        };
                        seller_stat_repo.create(&create_params).await?;
                    }

                    seller_id
                }
            };

            let gig_path = gig_to_scrape_url.path().to_owned();
            let gig_title = gig_page.get_title()?;
            log::info!("Gig title: {gig_title}");
            let gig_rating = gig_page.get_gig_rating()?;
            log::info!("Rating: {gig_rating}");
            let gig_reviews_count = gig_page.get_gig_reviews_count()?;
            log::info!("Reviews count: {gig_reviews_count}");
            let gig_description = gig_page.get_gig_description()?;
            log::info!("Description: {gig_description}");

            let create_params = db::gig_repo::CreateParams {
                path: gig_path,
                title: gig_title,
                rating: gig_rating,
                reviews_count: gig_reviews_count as i64,
                description: gig_description,
                page: last_gigs_page,
                seller_id,
                category_id: category_record.id,
            };
            let gig_id = gig_repo.create(&create_params).await?;

            log::info!("Metadata:");
            let gig_metadata = gig_page.get_gig_metadata()?;
            for metadata in gig_metadata {
                log::info!("\t {}: {}", metadata.name, metadata.values.join(", "));
                let create_params = db::gig_metadata_repo::CreateParams {
                    gig_id,
                    key: metadata.name,
                    values: metadata.values,
                };
                gig_metadata_repo.create(&create_params).await?;
            }

            let gallery_visuals = gig_page.get_gig_visuals()?;
            log::info!("Gallery visuals:");
            for visual in gallery_visuals {
                log::info!("\t {:?}: {}", visual.r#type(), visual.value());
                let visual_type_id = match visual_type_lookup.get_type_id(visual.r#type()).await? {
                    Some(visual_type_id) => visual_type_id,
                    None => visual_type_lookup.create(visual.r#type()).await?,
                };
                let create_params = db::visual_repo::CreateParams {
                    gig_id,
                    url: visual.value().to_owned(),
                    visual_type: visual_type_id,
                };
                visual_repo.create(&create_params).await?;
            }
            gig_page.close_visuals_modal()?;

            log::info!("Gig packages:");
            let gig_packages = gig_page.get_gig_packages()?;
            for package in gig_packages {
                log::info!(
                    "\t {} - {} @ {}",
                    package.r#type,
                    package.title,
                    package.price
                );
                let gig_package_type =
                    match gig_package_type_lookup.get_type_id(&package.r#type).await? {
                        Some(gig_package_type) => gig_package_type,
                        None => gig_package_type_lookup.create(&package.r#type).await?,
                    };
                let create_params = db::gig_package_repo::CreateParams {
                    r#type: gig_package_type,
                    price: package.price as f64,
                    title: package.title,
                    description: package.description,
                    gig_id,
                    delivery_time: package.delivery_time,
                };
                let package_id = gig_package_repo.create(&create_params).await?;
                let package_properties = package.properties;
                for (key, value) in package_properties.into_iter() {
                    log::info!("\t\t {}: {}", key, value);
                    let create_params = db::gig_package_feature_repo::CreateParams {
                        package_id,
                        key,
                        value,
                    };
                    gig_package_feature_repo.create(&create_params).await?;
                }
            }

            log::info!("Gig FAQs:");
            for result in gig_page.get_gig_faqs()? {
                let gig_faq = result?;
                log::info!("Question: {}\nAnswer: {}", gig_faq.question, gig_faq.answer);
                let create_params = db::gig_faq_repo::CreateParams {
                    gig_id,
                    question: gig_faq.question,
                    answer: gig_faq.answer,
                };
                gig_faq_repo.create(&create_params).await?;
            }

            log::info!("Gig reviews:");
            for (i, gig_review_result) in gig_page.get_gig_reviews()?.into_iter().enumerate() {
                if i == 80 {
                    gig_repo.set_scrape_completed(gig_id).await?;
                }
                let gig_review = gig_review_result?;
                log::info!("{:#?}", gig_review);
                let Ok(rating) = gig_review.rating.parse::<f64>() else {
                    log::error!(
                        "Error parsing the gig rating ({}) to a float value",
                        gig_review.rating
                    );
                    continue;
                };
                let (price_range_min, price_range_max) =
                    match PRICE_RANGE_PARSER.get_range_tuple(&gig_review.price) {
                        Ok(price_range) => price_range,
                        Err(e) => {
                            log::error!("{e}");
                            continue;
                        }
                    };
                let duration_string = STRING_CLEANER.as_simple_text(&gig_review.duration)?;
                let mut duration_parts = duration_string.split(" ");
                let duration_value = match STRING_CLEANER.as_usize(duration_parts.next().unwrap()) {
                    Ok(duration_value) => duration_value,
                    Err(e) => {
                        log::error!("{e}");
                        continue;
                    }
                };
                let duration_unit = duration_parts.next().unwrap().to_owned();
                let create_params = db::gig_review_repo::CreateParams {
                    gig_id,
                    country: gig_review.country,
                    rating,
                    price_range_min: price_range_min as i64,
                    price_range_max: price_range_max as i64,
                    duration_value: duration_value as i64,
                    duration_unit,
                    description: gig_review.description,
                };
                gig_review_repo.create(&create_params).await?;
            }
        }
    }
}
