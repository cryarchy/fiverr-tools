// Start Google Chrome with:
//     $ google-chrome --remote-debugging-port=9222 --user-data-dir="/tmp/chrome_dev"
// and update the websocket URL below

mod gig_data_extractor;
mod selector;
mod string_cleaner;

use anyhow::Result;
use gig_data_extractor::GigDataExtractor;
use headless_chrome::Browser;

#[tokio::main]
async fn main() -> Result<()> {
    // WebSocket debug URL - typically obtained from Chrome's DevTools or via CDP
    let debug_ws_url =
        "ws://127.0.0.1:9222/devtools/browser/5002c20c-1355-4fb6-add9-82f413072fc9".to_string();

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
        }
    }

    let fiverr_tab = fiverr_tab.unwrap();

    println!("Fiverr tab title: {}", fiverr_tab.get_title()?);

    let gig_data_extractor = GigDataExtractor::new((*fiverr_tab).clone());

    // println!("Gig title: {}", gig_data_extractor.extract_title()?);
    // println!("Rating: {}", gig_data_extractor.extract_gig_rating()?);
    // println!(
    //     "Reviews count: {}",
    //     gig_data_extractor.extract_gig_reviews_count()?
    // );
    // println!(
    //     "Description: {}",
    //     gig_data_extractor.extract_gig_description()?
    // );
    // println!("Metadata:");
    // let gig_metadata = gig_data_extractor.extract_gig_metadata()?;
    // for entry in gig_metadata {
    //     println!("\t {:?}", entry);
    // }
    // println!(
    //     "Seller rating: {}",
    //     gig_data_extractor.extract_seller_rating()?
    // );
    // println!(
    //     "Seller ratings count: {}",
    //     gig_data_extractor.extract_seller_ratings_count()?
    // );
    // println!(
    //     "Seller level: {}",
    //     gig_data_extractor.extract_seller_level()?
    // );
    // println!("Seller stats:");
    // let seller_stats = gig_data_extractor.extract_seller_stats()?;
    // for entry in seller_stats {
    //     println!("\t {:?}", entry);
    // }
    // println!(
    //     "Seller description: {}",
    //     gig_data_extractor.extract_seller_description()?
    // );
    // let gallery_visuals = gig_data_extractor.extract_gig_visuals()?;
    // println!("Gallery visuals:");
    // for entry in gallery_visuals {
    //     println!("\t {:?}", entry);
    // }
    // println!("Gig packages:");
    // let gig_packages = gig_data_extractor.extract_gig_packages()?;
    // println!("{:#?}", gig_packages);
    println!("Gig FAQs:");
    for result in gig_data_extractor.extract_gig_faqs()? {
        println!("{:#?}", result?);
    }
    // println!("Gig reviews:");
    // let gig_reviews = gig_data_extractor.extract_gig_reviews()?;
    // println!("{:#?}", gig_reviews);
    Ok(())
}
