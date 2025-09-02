mod app_config;

use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, anyhow};
use app_config::AppConfig;
use figment::{
    Figment,
    providers::{Format, Yaml},
};
use flexi_logger::Logger;
use headless_chrome::{Browser, Element, Tab};
use sqlx::{QueryBuilder, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use tokio::{fs, io::AsyncWriteExt, time::sleep};
use url::Url;
use uuid::Uuid;

static BTN_CLICK_WAIT_SECS: u64 = 1;
static BASE_URL: &str = "https://www.fiverr.com";

struct CustomBrowser {
    browser: Browser,
}

impl CustomBrowser {
    fn new(debug_ws_url: String, idle_browser_timeout: Duration) -> Result<Self> {
        let custom_browser = Self {
            browser: Browser::connect_with_timeout(debug_ws_url, idle_browser_timeout)?,
        };
        custom_browser.refresh()?;
        Ok(custom_browser)
    }

    fn get_fiverr_tab(&self) -> Result<Arc<Tab>> {
        let tabs = self.browser.get_tabs().lock().unwrap();

        let mut fiverr_tab = None;

        for tab in tabs.iter() {
            if tab.get_url().contains("fiverr") {
                fiverr_tab = Some(tab.clone());
                break;
            } else {
                tab.close(false)?;
            }
        }

        fiverr_tab.ok_or(anyhow!(
            "Could not get a tab with a url containing 'fiverr'"
        ))
    }

    fn refresh(&self) -> Result<()> {
        let tab = &self.browser.new_tab()?;
        tab.close(false)?;
        Ok(())
    }
}

struct FiverrNav<'a> {
    tab: &'a Arc<Tab>,
}

impl<'a> FiverrNav<'a> {
    fn new(tab: &'a Arc<Tab>) -> Self {
        Self { tab }
    }

    fn category_elements_selector() -> &'static str {
        "#categories-menu-package ul.categories li"
    }

    fn category_element_selector(idx: usize) -> String {
        format!("{} {}", Self::category_elements_selector(), idx)
    }

    fn menu_elements_selector() -> &'static str {
        "ul.menu-bucket"
    }

    fn scroll_right_btn_selector() -> &'static str {
        "#categories-menu-package button.right"
    }

    async fn scroll_right(&self) -> Result<()> {
        let button = self.tab.find_element(Self::scroll_right_btn_selector())?;
        button.click()?;
        sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        Ok(())
    }

    fn get_category_el(&'a self, category_id: &str) -> Result<Element<'a>> {
        let elements_selector = Self::category_elements_selector();
        log::info!("Find elements: {elements_selector}");
        let elements = self.tab.find_elements(elements_selector)?;
        let mut category_element = None;
        for (idx, element) in elements.into_iter().enumerate() {
            log::info!("Find element: {elements_selector} {idx} a");
            let anchor = element.find_element("a")?;
            log::info!("Get attribute: {elements_selector} {idx} a.href");
            let href = anchor.get_attribute_value("href")?.ok_or(anyhow!(format!(
                "Element ({}) does not have a 'href' attribute",
                Self::category_element_selector(idx)
            )))?;
            if href.contains(category_id) {
                category_element = Some(element);
                break;
            }
        }

        category_element.ok_or(anyhow!(format!(
            "Could not find category element '{}'",
            category_id
        )))
    }

    async fn scroll_category_el_into_view(&'a self, category_id: &str) -> Result<Element<'a>> {
        loop {
            let category_el = self.get_category_el(category_id)?;
            log::info!("Get attribute: [{category_id}].style");
            let style = category_el.get_attribute_value("style")?.ok_or(anyhow!(
                "Category element ({category_id}) does not have a 'style' attribute"
            ))?;
            if style.contains("none") {
                self.scroll_right().await?;
            } else {
                break Ok(category_el);
            }
        }
    }

    fn get_menu_el<'b>(category_el: Element<'b>, menu_id: &str) -> Result<Element<'b>> {
        let elements_selector = Self::menu_elements_selector();
        log::info!("Find elements: [el:category] {elements_selector}");
        let elements = category_el.find_elements(elements_selector)?;
        let mut menu_item_element = None;
        for (menu_idx, menu_element) in elements.into_iter().enumerate() {
            log::info!("Find elements: [el:category] {elements_selector}.nth-child({menu_idx}) li");
            let menu_items = menu_element.find_elements("li")?;
            for (menu_item_idx, element) in menu_items.into_iter().enumerate() {
                log::info!("Find element: [el:li] {menu_item_idx} a");
                let anchor = element.find_element("a")?;
                log::info!("Get attribute: [el:li] {menu_item_idx} a");
                let href = anchor.get_attribute_value("href")?.ok_or(anyhow!(format!(
                    "Element ([el:li] {menu_item_idx} a) does not have a 'href' attribute",
                )))?;
                log::debug!("{href}");
                if href.contains(menu_id) {
                    menu_item_element = Some(element);
                    break;
                }
            }
            if menu_item_element.is_some() {
                break;
            }
        }

        menu_item_element.ok_or(anyhow!(format!(
            "Could not find category element '{}'",
            menu_id
        )))
    }

    async fn go_to(&self, category_id: &str, menu_id: &str) -> Result<()> {
        let category_el = self.scroll_category_el_into_view(category_id).await?;
        log::info!("Mouse over: [{category_id}].style");
        category_el.move_mouse_over()?;
        log::info!("Wait for element: [{category_id}] .menu-bucket");
        category_el.wait_for_element(".menu-bucket")?;
        let menu_item_el = Self::get_menu_el(category_el, menu_id)?;
        menu_item_el.click()?;
        self.tab.wait_until_navigated()?;
        Ok(())
    }
}

struct ScrapedGigsStore {
    db: SqlitePool,
}

impl ScrapedGigsStore {
    fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    async fn is_scraped(&self, gig_url: &str) -> Result<bool> {
        let record = sqlx::query!("SELECT count(*) as cnt FROM gigs WHERE url = $1", gig_url)
            .fetch_one(&self.db)
            .await?;
        Ok(record.cnt > 0)
    }

    async fn save_visuals(&self, gig_id: String, visuals: Vec<VisualData>) -> Result<()> {
        log::debug!("{:#?}", visuals);
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO visuals(id, gig_id, file_path, visual_type)");

        let visuals = visuals
            .into_iter()
            .map(|visual| {
                let id = Uuid::new_v4().to_string();
                (id, gig_id.to_owned(), visual.url, visual.typ.to_string())
            })
            .collect::<Vec<_>>();

        query_builder.push_values(visuals, |mut b, (id, gig_id, file_path, visual_type)| {
            b.push_bind(id)
                .push_bind(gig_id)
                .push_bind(file_path)
                .push_bind(visual_type);
        });

        let query = query_builder.build();
        query.execute(&self.db).await?;

        Ok(())
    }

    async fn save(&self, gig: GigData) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO gigs(id, url, title, description, page) VALUES($1, $2, $3, $4, $5)",
            id,
            gig.url,
            gig.title,
            gig.description,
            gig.page
        )
        .execute(&self.db)
        .await?;
        self.save_visuals(id, gig.visuals).await?;

        Ok(())
    }

    async fn last_scraped_page(&self) -> Result<u32> {
        let fetch_result = sqlx::query!("SELECT page FROM gigs ORDER BY page DESC LIMIT 1")
            .fetch_one(&self.db)
            .await;
        match fetch_result {
            Err(sqlx::Error::RowNotFound) => Ok(1),
            Ok(record) => Ok(record.page as u32),
            Err(e) => Err(e.into()),
        }
    }
}

struct MenuItemPage<'a> {
    tab: &'a Arc<Tab>,
    store: Arc<ScrapedGigsStore>,
}

impl<'a> MenuItemPage<'a> {
    fn new(tab: &'a Arc<Tab>, store: Arc<ScrapedGigsStore>) -> Self {
        Self { tab, store }
    }

    fn gig_cards_selector() -> &'static str {
        "#main-wrapper .basic-gig-card"
    }

    fn get_gig_cards(&'a self) -> Result<Vec<Element<'a>>> {
        let elements_selector = Self::gig_cards_selector();
        log::info!("Wait for elements: {elements_selector}");
        self.tab.wait_for_elements(elements_selector)
    }

    fn get_gig_orders_count<'b>(gig_card: &Element<'b>) -> Result<usize> {
        log::info!("Find element: [ref:gig_card] .orca-rating > span");
        let span = gig_card.find_element(".orca-rating > span")?;
        log::info!("Get inner text: [ref:gig_card] .orca-rating > span");
        let orders_count: usize = span
            .get_inner_text()?
            .replace(['(', ')'], "")
            .trim()
            .parse()
            .unwrap_or(1000);
        Ok(orders_count)
    }

    fn gig_order_count_gt_threshold<'b>(gig_card: &Element<'b>) -> Result<bool> {
        let orders_count = Self::get_gig_orders_count(gig_card)?;
        Ok(orders_count >= 100)
    }

    fn visit_gig<'b>(&self, gig_card: Element<'b>) -> Result<()> {
        log::info!("Find element: [ref:gig_card] a");
        let anchor = gig_card.find_element("a")?;
        anchor.click()?;
        self.tab.wait_until_navigated()?;
        Ok(())
    }

    fn get_gig_url<'b>(gig_card: &Element<'b>) -> Result<String> {
        log::info!("Find element: [ref:gig_card] a");
        let anchor = gig_card.find_element("a")?;
        log::info!("Get attribute: [el:gig_card] a");
        let href = anchor.get_attribute_value("href")?.ok_or(anyhow!(
            "Element ([el:gig_card] a) does not have a 'href' attribute",
        ))?;
        let stripped_url = QueryPathStripper::strip(&href);
        UrlNormalizer::normalize(stripped_url)
    }

    fn next_gigs_page_btn_selector() -> &'static str {
        r#"#main-wrapper .main-content a[aria-label="Next"][role="link"]"#
    }

    async fn next_gigs_page(&self) -> Result<()> {
        let element_selector = Self::next_gigs_page_btn_selector();
        log::info!("Wait for element: {element_selector}");
        let next_page_btn = self.tab.wait_for_element(element_selector)?;
        log::info!("Click: {element_selector}");
        next_page_btn.click()?;
        self.tab.wait_until_navigated()?;
        sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        Ok(())
    }

    fn prev_gigs_page_btn_selector() -> &'static str {
        r#"#main-wrapper .main-content a[aria-label="Previous"][role="link"]"#
    }

    async fn prev_gigs_page(&self) -> Result<()> {
        let element_selector = Self::prev_gigs_page_btn_selector();
        log::info!("Wait for element: {element_selector}");
        let prev_page_btn = self.tab.wait_for_element(element_selector)?;
        log::info!("Click: {element_selector}");
        prev_page_btn.click()?;
        self.tab.wait_until_navigated()?;
        sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        Ok(())
    }

    fn get_page_count(&self) -> Result<u32> {
        let page_url = self.tab.get_url();
        let page_url = Url::parse(&page_url)?;
        let page_count = page_url
            .query_pairs()
            .find_map(|(key, value)| match key == "page" {
                true => Some(value.to_string()),
                false => None,
            })
            .unwrap_or("1".to_string());
        let page_count: u32 = page_count.parse()?;
        Ok(page_count)
    }

    async fn go_to_page(&self, page: u32) -> Result<()> {
        loop {
            let current_page = self.get_page_count()?;
            log::debug!("current page: {current_page}");
            match current_page.cmp(&page) {
                Ordering::Equal => break Ok(()),
                Ordering::Greater => {
                    self.prev_gigs_page().await?;
                }
                Ordering::Less => {
                    self.next_gigs_page().await?;
                }
            }
        }
    }

    async fn go_to_new_gig(&self) -> Result<u32> {
        loop {
            let gig_cards = self.get_gig_cards()?;
            for (idx, card) in gig_cards.into_iter().enumerate() {
                log::info!("Gig index: {idx}");
                if Self::gig_order_count_gt_threshold(&card)? {
                    let gig_url = Self::get_gig_url(&card)?;
                    log::debug!("Gig URL: {gig_url}");
                    log::debug!("is scraped: {}", self.store.is_scraped(&gig_url).await?);
                    if self.store.is_scraped(&gig_url).await? {
                        log::debug!("continuing...");
                        continue;
                    }
                    self.visit_gig(card)?;
                    return self.get_page_count();
                }
            }
            self.next_gigs_page().await?;
        }
    }
}

struct GigPage<'a> {
    tab: &'a Arc<Tab>,
    page: u32,
}

#[derive(Debug)]
enum SlideType {
    Video,
    Image,
    Pdf,
}

impl std::fmt::Display for SlideType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            SlideType::Image => "image",
            SlideType::Video => "video",
            SlideType::Pdf => "pdf",
        };
        f.write_str(v)
    }
}

#[derive(Debug)]
struct VisualData {
    url: String,
    typ: SlideType,
}

struct GigData {
    url: String,
    title: String,
    description: String,
    visuals: Vec<VisualData>,
    page: u32,
}

impl<'a> GigPage<'a> {
    fn new(tab: &'a Arc<Tab>, page: u32) -> Self {
        Self { tab, page }
    }

    fn title_selector() -> &'static str {
        "#main-wrapper .gig-page .gig-overview h1"
    }

    fn about_selector() -> &'static str {
        "#main-wrapper .gig-page .description-content"
    }

    fn get_url(&self) -> Result<String> {
        let url = self.tab.get_url();
        let stripped_url = QueryPathStripper::strip(&url);
        UrlNormalizer::normalize(stripped_url)
    }

    fn get_about(&self) -> Result<String> {
        let element_selector = Self::about_selector();
        log::info!("Find element: {element_selector}");
        let description_el = self.tab.find_element(element_selector)?;
        log::info!("Get content: {element_selector}");
        let description = description_el.get_content()?;
        Ok(description)
    }

    fn get_title(&self) -> Result<String> {
        let element_selector = Self::title_selector();
        log::info!("Find element: {element_selector}");
        let title_el = self.tab.find_element(element_selector)?;
        log::info!("Get inner text: {element_selector}");
        let title = title_el.get_inner_text()?;
        Ok(title)
    }

    fn current_slide_selector() -> &'static str {
        "#main-wrapper .gig-page .gallery-slideshow .slideshow-slide.current .slide"
    }

    fn next_slide_selector() -> &'static str {
        "#main-wrapper .gig-page .gallery-slideshow .nav-next"
    }

    fn current_gallery_slide_selector() -> &'static str {
        ".gallery-modal .slideshow-slide.current .slide"
    }

    fn gallery_modal_selector() -> &'static str {
        ".gallery-modal"
    }

    fn gallery_next_slide_btn_selector() -> &'static str {
        ".gallery-modal .modal-nav-next"
    }

    fn gallery_close_btn_selector() -> &'static str {
        ".gallery-modal .modal-close"
    }

    fn get_slide_type<'b>(slide_el: &Element<'b>) -> Result<SlideType> {
        log::info!("Get attribute value: [ref:slide]");
        let class = slide_el.get_attribute_value("class")?.ok_or(anyhow!(
            "Element ([ref:slide] does not have the attribute 'class')"
        ))?;
        if class.contains("video") {
            Ok(SlideType::Video)
        } else if class.contains("image") {
            Ok(SlideType::Image)
        } else {
            Ok(SlideType::Pdf)
        }
    }

    async fn switch_to_next_slide(&self) -> Result<()> {
        let element_selector = Self::next_slide_selector();
        log::info!("Find element: {element_selector}");
        let next_btn = self.tab.find_element(element_selector)?;
        log::info!("Click: {element_selector}");
        next_btn.click()?;
        sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        Ok(())
    }

    async fn close_gallery(&self) -> Result<()> {
        let element_selector = Self::gallery_close_btn_selector();
        log::info!("Find element: {element_selector}");
        let close_btn = self.tab.find_element(element_selector)?;
        log::info!("Click: {element_selector}");
        close_btn.click()?;
        sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        Ok(())
    }

    async fn switch_to_next_gallery_slide(&self) -> Result<()> {
        let element_selector = Self::gallery_next_slide_btn_selector();
        log::info!("Find element: {element_selector}");
        if let Ok(next_btn) = self.tab.find_element(element_selector) {
            log::info!("Click: {element_selector}");
            next_btn.click()?;
            sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        }
        Ok(())
    }

    async fn open_slideshow(&self) -> Result<()> {
        loop {
            let element_selector = Self::gallery_modal_selector();
            log::info!("Find element: {element_selector}");
            let gallery_modal = self.tab.find_element(element_selector);
            if gallery_modal.is_ok() {
                break Ok(());
            }
            let element_selector = Self::current_slide_selector();
            log::info!("Find element: {element_selector}");
            let current_slide_el = self.tab.find_element(element_selector)?;
            match Self::get_slide_type(&current_slide_el)? {
                SlideType::Image => {
                    log::info!("Click: {element_selector}");
                    current_slide_el.click()?;
                }
                _ => {
                    self.switch_to_next_slide().await?;
                }
            }
        }
    }

    async fn get_visuals(&self) -> Result<Vec<(String, SlideType)>> {
        self.open_slideshow().await?;
        let mut visuals = Vec::new();
        let visual_exists = |visuals: &mut Vec<(String, SlideType)>, key: &str| {
            visuals.iter().any(|(url, _)| url == key)
        };
        loop {
            let element_selector = Self::current_gallery_slide_selector();
            log::info!("Find element: {element_selector}");
            let current_slide = self.tab.find_element(element_selector)?;
            let slide_type = Self::get_slide_type(&current_slide)?;
            match slide_type {
                SlideType::Image => {
                    log::info!("Find element: {element_selector} img");
                    let image_el = current_slide.find_element("img")?;
                    log::info!("Get attribute value: {element_selector} img.src");
                    let source = image_el.get_attribute_value("src")?.ok_or(anyhow!(
                        "Element ({element_selector} img) has no attribute 'src'."
                    ))?;
                    if visual_exists(&mut visuals, &source) {
                        break;
                    }
                    visuals.push((source, SlideType::Image));
                }
                SlideType::Video => {
                    log::info!("Find element: {element_selector} button");
                    let play_btn = current_slide.find_element("button")?;
                    log::info!("Click: {element_selector} button");
                    play_btn.click()?;
                    log::info!("Wait for element: {element_selector} video");
                    let video_el = current_slide.wait_for_element("video")?;
                    log::info!("Get attribute value: {element_selector} video.src");
                    let source = video_el.get_attribute_value("src")?.ok_or(anyhow!(
                        "Element ({element_selector} video) has no attribute 'src'."
                    ))?;
                    if visual_exists(&mut visuals, &source) {
                        break;
                    }
                    visuals.push((source, SlideType::Video));
                }
                _ => (),
            }
            self.switch_to_next_gallery_slide().await?;
        }
        self.close_gallery().await?;
        Ok(visuals)
    }

    async fn scrape(&self) -> Result<GigData> {
        let url = self.get_url()?;
        let description = self.get_about()?;
        let title = self.get_title()?;
        ModalCloser::close_open_modal(self.tab).await?;
        let visuals = self
            .get_visuals()
            .await?
            .into_iter()
            .map(|visual| VisualData {
                url: visual.0,
                typ: visual.1,
            })
            .collect();
        Ok(GigData {
            url,
            title,
            description,
            visuals,
            page: self.page,
        })
    }
}

struct ModalCloser {}

impl ModalCloser {
    fn open_modal_close_btn_selector() -> &'static str {
        r#"article[aria-modal="true"][role="dialog"] button:has(svg)"#
    }

    async fn close_open_modal(tab: &Arc<Tab>) -> Result<()> {
        let element_selector = Self::open_modal_close_btn_selector();
        log::info!("Find element: {element_selector}");
        if let Ok(modal_close_btn) = tab.find_element(element_selector) {
            log::info!("Click: {element_selector}");
            modal_close_btn.click()?;
            sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        }
        Ok(())
    }
}

struct ErrorPageDetector {}

impl ErrorPageDetector {
    fn error_code_selector() -> &'static str {
        "body > main > article > code"
    }

    fn figcaption_selector() -> &'static str {
        "body > main > figure > figcaption > h1"
    }

    fn is_error_page(tab: &Arc<Tab>) -> Result<bool> {
        let element_selector = Self::error_code_selector();
        log::info!("Find element: {element_selector}");
        if let Ok(code_el) = tab.find_element(element_selector) {
            log::info!("Get inner text: {element_selector}");
            let text = code_el.get_inner_text()?;
            if text.contains("ERRCODE") {
                return Ok(true);
            }
        }

        let element_selector = Self::figcaption_selector();
        log::info!("Find element: {element_selector}");
        if let Ok(caption_el) = tab.find_element(element_selector) {
            log::info!("Get inner text: {element_selector}");
            let text = caption_el.get_inner_text()?;
            if text.contains("no time") {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn process(tab: &Arc<Tab>) -> Result<()> {
        let element_selector = Self::error_code_selector();
        log::info!("Find element: {element_selector}");
        if Self::is_error_page(tab)? {
            log::info!("Reload tab");
            tab.reload(true, None)?;
            sleep(Duration::from_secs(BTN_CLICK_WAIT_SECS)).await;
        }
        Ok(())
    }
}

struct QueryPathStripper {}

impl QueryPathStripper {
    fn strip(url: &str) -> &str {
        url.split("?").next().unwrap()
    }
}

struct UrlNormalizer {}

impl UrlNormalizer {
    fn normalize(url: &str) -> Result<String> {
        if url.trim().starts_with('/') {
            let mut parsed_url = Url::parse(BASE_URL)?;
            parsed_url.set_path(url);
            Ok(parsed_url.into())
        } else {
            let url = Url::parse(url)?;
            Ok(url.into())
        }
    }
}

struct ResourceDownloader {
    download_dir: PathBuf,
}

impl ResourceDownloader {
    async fn new(download_dir: &str) -> Result<Self> {
        let download_dir = Path::new(download_dir).to_path_buf();
        fs::create_dir_all(&download_dir).await?;
        Ok(Self { download_dir })
    }

    pub async fn download_media_files(&self, visuals: Vec<VisualData>) -> Result<Vec<VisualData>> {
        let client = reqwest::Client::new();
        let mut results = Vec::new();

        for visual in visuals {
            let file_path = self.download_single_file(&client, &visual.url).await?;
            let file_path_str = file_path
                .to_str()
                .ok_or(anyhow!("Encountered path without string"))?
                .to_owned();
            results.push(VisualData {
                url: file_path_str,
                typ: visual.typ,
            });
        }

        Ok(results)
    }

    async fn download_single_file(&self, client: &reqwest::Client, uri: &str) -> Result<PathBuf> {
        // Parse and validate URL
        let url = Url::parse(uri)?;

        // Extract file extension from URL path
        let path = url.path();
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(".mp4")
            .to_lowercase();

        // Generate UUID for new filename
        let uuid = Uuid::new_v4();
        let uuid_filename = format!("{}.{}", uuid, extension);

        // Download the file
        let response = client.get(uri).send().await?;

        if !response.status().is_success() {
            return Err(response.error_for_status().unwrap_err().into());
        }

        let bytes = response.bytes().await?;

        // Write to file
        let file_path = self.download_dir.join(&uuid_filename);
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        Ok(file_path)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_config: AppConfig = Figment::new()
        .merge(Yaml::file("app-config.yaml"))
        .extract()?;

    Logger::try_with_str(app_config.log_level)?.start()?;

    let connection_options = SqliteConnectOptions::from_str(&app_config.database_url)
        .unwrap()
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(connection_options).await?;
    let gigs_store = Arc::new(ScrapedGigsStore::new(db_pool));

    let resource_downloader = ResourceDownloader::new(&app_config.download_dir).await?;

    let browser = CustomBrowser::new(app_config.browser_ws_url, Duration::from_secs(600))?;

    let mut fiverr_tab = browser.get_fiverr_tab()?;
    log::info!("Fiverr tab title: {}", fiverr_tab.get_title()?);

    ModalCloser::close_open_modal(&fiverr_tab).await?;

    loop {
        ErrorPageDetector::process(&fiverr_tab).await?;
        let fiverr_nav = FiverrNav::new(&fiverr_tab);

        fiverr_nav.go_to("programming-tech", "business").await?;
        // fiverr_nav.go_to("data", "text-analysis-nlp")?;

        let last_scraped_page = gigs_store.last_scraped_page().await?;

        let menu_item_page = MenuItemPage::new(&fiverr_tab, gigs_store.clone());
        menu_item_page.go_to_page(last_scraped_page).await?;
        let gig_page = menu_item_page.go_to_new_gig().await?;

        // close current tab since new tab is opened when a gig card is clicked.

        fiverr_tab.close(true)?;

        sleep(Duration::from_secs(5)).await;

        fiverr_tab = browser.get_fiverr_tab()?;

        let gig_page = GigPage::new(&fiverr_tab, gig_page);
        let gig_data = gig_page.scrape().await?;
        let visuals = resource_downloader
            .download_media_files(gig_data.visuals)
            .await?;
        log::debug!("Gig URL: {}", gig_data.url);
        let gig_data = GigData {
            url: gig_data.url,
            title: gig_data.title,
            description: gig_data.description,
            page: gig_data.page,
            visuals,
        };
        gigs_store.save(gig_data).await?;
    }

    Ok(())
}
