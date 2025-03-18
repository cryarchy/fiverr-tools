// Start Google Chrome with:
//     $ google-chrome --remote-debugging-port=9222 --user-data-dir="/tmp/chrome_dev"
// and update the websocket URL below

mod categories_menu;
mod gig_page;
mod gigs_page;
mod markup_interaction_error;
mod selector;
mod site_nav;
mod string_cleaner;
mod wrapped_element;

use std::{thread::sleep, time::Duration};

use anyhow::Result;
use categories_menu::CategoriesMenu;
use gig_page::GigPage;
use gigs_page::GigsPage;
use headless_chrome::Browser;
use site_nav::SiteNav;

#[tokio::main]
async fn main() -> Result<()> {
    // WebSocket debug URL - typically obtained from Chrome's DevTools or via CDP
    let debug_ws_url =
        "ws://127.0.0.1:9222/devtools/browser/f848262a-6a8d-4c09-8faf-470b1ab025f6".to_string();

    // Connect to an existing browser using its WebSocket debug URL
    let browser = Browser::connect(debug_ws_url)?;

    let tab = browser.new_tab()?;
    tab.close(false)?;

    // Get the list of existing tabs/pages
    let tabs = browser.get_tabs().lock().unwrap();

    let mut fiverr_tab = None;

    for tab in tabs.iter() {
        if tab.get_url().contains("fiverr") {
            fiverr_tab = Some(tab);
            break;
        }
    }

    let fiverr_tab = fiverr_tab.unwrap();

    println!("Fiverr tab title: {}", fiverr_tab.get_title()?);

    // let page_nav = SiteNav::new((*fiverr_tab).clone());

    // page_nav.go_home()?;

    // let categories_menu = CategoriesMenu::new((*fiverr_tab).clone());
    // for category in categories_menu.get_gig_categories()? {
    //     let category = category?;
    //     println!("{}", category.main_category);
    //     println!("- {}", category.category_group);
    //     println!("    - {} ({})", category.name, category.url);
    // }

    let gigs_page = GigsPage::new((*fiverr_tab).clone());
    // gigs_page.go_to_page(3)?;
    println!("Current page: {}", gigs_page.get_current_page()?);
    for gig_card in gigs_page.gigs()? {
        println!("- {}", gig_card?.url);
    }

    // let gig_page = GigPage::new((*fiverr_tab).clone());

    // println!("Gig title: {}", gig_page.get_title()?);
    // println!("Rating: {}", gig_page.get_gig_rating()?);
    // println!("Reviews count: {}", gig_page.get_gig_reviews_count()?);
    // println!("Description: {}", gig_page.get_gig_description()?);
    // println!("Metadata:");
    // let gig_metadata = gig_page.get_gig_metadata()?;
    // for entry in gig_metadata {
    //     println!("\t {:?}", entry);
    // }
    // println!("Seller rating: {}", gig_page.get_seller_rating()?);
    // println!(
    //     "Seller ratings count: {}",
    //     gig_page.get_seller_ratings_count()?
    // );
    // println!("Seller level: {}", gig_page.get_seller_level()?);
    // println!("Seller stats:");
    // let seller_stats = gig_page.get_seller_stats()?;
    // for entry in seller_stats {
    //     println!("\t {:?}", entry);
    // }
    // println!("Seller description: {}", gig_page.get_seller_description()?);
    // let gallery_visuals = gig_page.get_gig_visuals()?;
    // println!("Gallery visuals:");
    // for entry in gallery_visuals {
    //     println!("\t {:?}", entry);
    // }
    // println!("Gig packages:");
    // let gig_packages = gig_page.get_gig_packages()?;
    // println!("{:#?}", gig_packages);
    // println!("Gig FAQs:");
    // for result in gig_page.get_gig_faqs()? {
    //     println!("{:#?}", result?);
    // }
    // println!("Gig reviews:");
    // for gig_review_result in gig_page.get_gig_reviews()? {
    //     println!("{:#?}", gig_review_result?);
    //     break;
    // }
    Ok(())
}
