use std::{thread::sleep, time::Duration};

use crate::selector::Selector;

use super::GigPage;

#[derive(Debug)]
pub enum GalleryVisual {
    Video(String),
    Image(String),
}

impl GigPage {
    pub fn get_gig_visuals(&self) -> Result<Vec<GalleryVisual>, crate::Error> {
        let close_button_selector = Selector::new(".modal-package .modal-close".to_owned());
        if let Ok(close_button_el) = self.tab.find_element(&close_button_selector) {
            close_button_el.click()?;
        }

        let gallery_thumbnail_els_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails a.thumbnail"
                .to_owned(),
        );
        let gallery_thumbnail_els_count = self
            .tab
            .find_elements(&gallery_thumbnail_els_selector)?
            .len();

        let mut gallery_visuals = Vec::new();
        for index in 1..=gallery_thumbnail_els_count {
            let thumbnail_el_selector = gallery_thumbnail_els_selector.nth_child(index);
            let gallery_thumbnail_el = self.tab.find_element(&thumbnail_el_selector)?;
            let class_name = gallery_thumbnail_el.get_expected_attribute_value("class")?;
            log::debug!("{}", class_name);
            if !class_name.contains("active") {
                // rewind the gallery thumbnails container to the first item
                let previous_button_selector = Selector::new(
                    "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .nav-prev"
                        .to_owned(),
                );
                while let Ok(previous_button) = self.tab.find_element(&previous_button_selector) {
                    previous_button.click()?;
                    sleep(Duration::from_millis(100));
                }

                let active_thumbnail_selector = Selector::new(format!(
                    "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .thumbs-container a.thumbnail:nth-child({index}).active"
                ));
                for i in 0..gallery_thumbnail_els_count {
                    self.tab.find_element(&thumbnail_el_selector)?.click()?;
                    match self
                        .tab
                        .wait_for_element(&active_thumbnail_selector)
                        .is_ok()
                    {
                        true => break,
                        false => {
                            log::debug!(
                                "Gallery thumbnail at index {index} not visible. Clicking next button."
                            );
                            let next_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .nav-next".to_owned());
                            self.tab.find_element(&next_button_selector)?.click()?;
                            sleep(Duration::from_millis(100));
                            if i == gallery_thumbnail_els_count {
                                return Err(crate::Error::Unexpected(
                                    "Waiting for the clicked thumbnail to become 'active' failed!"
                                        .to_owned(),
                                ));
                            }
                        }
                    }
                }
            }

            let current_visual_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide".to_owned());
            let current_visual_el = self.tab.find_element(&current_visual_selector)?;
            let class_name = current_visual_el.get_expected_attribute_value("class")?;
            if class_name.contains("slide-video") {
                let load_video_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide button".to_owned());
                self.tab
                    .find_element(&load_video_button_selector)?
                    .click()?;

                let video_el_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide video".to_owned());
                let video_el = self.tab.wait_for_element(&video_el_selector)?;
                let video_src = video_el.get_expected_attribute_value("src")?;
                gallery_visuals.push(GalleryVisual::Video(video_src));
            } else {
                let modal_slideshow_el_selector =
                    Selector::new(".modal-package .slideshow-component".to_owned());
                let modal_slideshow_el = self.tab.find_element(&modal_slideshow_el_selector);
                if modal_slideshow_el.is_err() {
                    let anchor_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide figure".to_owned());
                    self.tab.find_element(&anchor_selector)?.click()?;
                    self.tab.wait_for_element(&modal_slideshow_el_selector)?;
                }
                gallery_visuals.clear();

                let modal_slides_selector = Selector::new(
                    ".modal-package .slideshow-component .slideshow-slide".to_owned(),
                );
                let modal_slides_count = self.tab.find_elements(&modal_slides_selector)?.len();
                // go to the first slide
                let first_slide_selector = Selector::new(
                    ".modal-package .slideshow-component .slideshow-slide:nth-child(1).current"
                        .to_owned(),
                );
                log::debug!("Navigating to the first slide.");
                while self.tab.find_element(&first_slide_selector).is_err() {
                    let modal_prev_button_selector =
                        Selector::new(".modal-package .modal-nav-prev".to_owned());
                    self.tab
                        .find_element(&modal_prev_button_selector)?
                        .click()?;
                    sleep(Duration::from_millis(100));
                }
                for index in 1..=modal_slides_count {
                    let current_slide_selector = Selector::new(format!(
                        ".modal-package .slideshow-component .slideshow-slide:nth-child({index}).current .slide"
                    ));
                    let current_slide_el = self.tab.find_element(&current_slide_selector)?;
                    let class_name = current_slide_el.get_expected_attribute_value("class")?;
                    if class_name.contains("slide-video") {
                        let button_selector = current_slide_selector.append("button");
                        self.tab.find_element(&button_selector)?.click()?;
                        let video_el_selector = Selector::new(format!(
                            ".modal-package .slideshow-component .slideshow-slide:nth-child({index}).current .slide video"
                        ));
                        let video_el = self.tab.wait_for_element(&video_el_selector)?;
                        let video_src = video_el.get_expected_attribute_value("src")?;
                        gallery_visuals.push(GalleryVisual::Video(video_src));
                    } else {
                        let slide_img_selector = current_slide_selector.append("img");
                        let current_slide_image_el = self.tab.find_element(&slide_img_selector)?;
                        let image_src =
                            current_slide_image_el.get_expected_attribute_value("src")?;
                        gallery_visuals.push(GalleryVisual::Image(image_src));
                    }

                    log::debug!("Navigating to the next slide.");
                    let modal_next_button_selector =
                        Selector::new(".modal-package .modal-nav-next".to_owned());
                    self.tab
                        .find_element(&modal_next_button_selector)?
                        .click()?;
                    sleep(Duration::from_millis(100));
                }
                return Ok(gallery_visuals);
            }
        }
        Ok(gallery_visuals)
    }
}
