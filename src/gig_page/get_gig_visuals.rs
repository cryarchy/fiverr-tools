use std::{thread::sleep, time::Duration};

use crate::{markup_interaction_error::MarkupInteractionError, selector::Selector};

use super::GigPage;

#[derive(Debug, thiserror::Error)]
pub enum GigVisualsError {
    #[error("GigVisualsError: {0}")]
    MarkupInteractionError(#[from] MarkupInteractionError),
    #[error("GigVisualsError: Class attribute not found for '{0}'")]
    ClassAttributeNotFound(String),
    #[error("GigVisualsError: Src attribute not found for '{0}'")]
    SrcAttributeNotFound(String),
    #[error("GigVisualsError: Unexpected '{0}'")]
    Unexpected(String),
}

#[derive(Debug)]
pub enum GalleryVisual {
    Video(String),
    Image(String),
}

impl GigPage {
    pub fn get_gig_visuals(&self) -> Result<Vec<GalleryVisual>, GigVisualsError> {
        let close_button_selector = Selector::new(".modal-package .modal-close".to_owned());
        if let Ok(close_button_el) = self.tab.find_element(close_button_selector.as_ref()) {
            close_button_el
                .click()
                .map_err(Self::map_err_fn(close_button_selector))?;
        }

        let gallery_thumbnail_els_selector = Selector::new(
            "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails a.thumbnail"
                .to_owned(),
        );
        let gallery_thumbnail_els_count = self
            .tab
            .find_elements(gallery_thumbnail_els_selector.as_ref())
            .map_err(Self::map_err_fn(gallery_thumbnail_els_selector.to_owned()))?
            .len();

        let mut gallery_visuals = Vec::new();
        for index in 1..=gallery_thumbnail_els_count {
            let thumbnail_el_selector = gallery_thumbnail_els_selector.nth_child(index);
            let gallery_thumbnail_el = self
                .tab
                .find_element(thumbnail_el_selector.as_ref())
                .map_err(Self::map_err_fn(thumbnail_el_selector.to_owned()))?;
            let class_name = gallery_thumbnail_el
                .get_attribute_value("class")
                .map_err(Self::map_err_fn(thumbnail_el_selector.to_owned()))?
                .ok_or(GigVisualsError::ClassAttributeNotFound(
                    thumbnail_el_selector.to_string(),
                ))?;
            println!("{}", class_name);
            if !class_name.contains("active") {
                // rewind the gallery thumbnails container to the first item
                let previous_button_selector = Selector::new(
                    "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .nav-prev"
                        .to_owned(),
                );
                while let Ok(previous_button) =
                    self.tab.find_element(previous_button_selector.as_ref())
                {
                    previous_button
                        .click()
                        .map_err(Self::map_err_fn(previous_button_selector.to_owned()))?;
                    sleep(Duration::from_millis(100));
                }

                let active_thumbnail_selector = Selector::new(format!(
                    "#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .thumbs-container a.thumbnail:nth-child({index}).active"
                ));
                for i in 0..gallery_thumbnail_els_count {
                    self.tab
                        .find_element(thumbnail_el_selector.as_ref())
                        .map_err(Self::map_err_fn(thumbnail_el_selector.to_owned()))?
                        .click()
                        .map_err(Self::map_err_fn(thumbnail_el_selector.to_owned()))?;
                    match self
                        .tab
                        .wait_for_element(active_thumbnail_selector.as_ref())
                        .is_ok()
                    {
                        true => break,
                        false => {
                            println!(
                                "Gallery thumbnail at index {index} not visible. Clicking next button."
                            );
                            let next_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-thumbnails .nav-next".to_owned());
                            self.tab
                                .find_element(next_button_selector.as_ref())
                                .map_err(Self::map_err_fn(next_button_selector.to_owned()))?
                                .click()
                                .map_err(Self::map_err_fn(next_button_selector))?;
                            sleep(Duration::from_millis(100));
                            if i == gallery_thumbnail_els_count {
                                return Err(GigVisualsError::Unexpected(
                                    "Waiting for the clicked thumbnail to become 'active' failed!"
                                        .to_owned(),
                                ));
                            }
                        }
                    }
                }
            }

            let current_visual_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide".to_owned());
            let current_visual_el = self
                .tab
                .find_element(current_visual_selector.as_ref())
                .map_err(Self::map_err_fn(current_visual_selector.to_owned()))?;
            let class_name = current_visual_el
                .get_attribute_value("class")
                .map_err(Self::map_err_fn(current_visual_selector.to_owned()))?
                .ok_or(GigVisualsError::ClassAttributeNotFound(
                    current_visual_selector.to_string(),
                ))?;
            if class_name.contains("slide-video") {
                let load_video_button_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide button".to_owned());
                self.tab
                    .find_element(load_video_button_selector.as_ref())
                    .map_err(Self::map_err_fn(load_video_button_selector.to_owned()))?
                    .click()
                    .map_err(Self::map_err_fn(load_video_button_selector))?;

                let video_el_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide video".to_owned());
                let video_el = self
                    .tab
                    .wait_for_element(video_el_selector.as_ref())
                    .map_err(Self::map_err_fn(video_el_selector.to_owned()))?;
                let video_src = video_el
                    .get_attribute_value("src")
                    .map_err(Self::map_err_fn(video_el_selector.to_owned()))?
                    .ok_or(GigVisualsError::SrcAttributeNotFound(
                        video_el_selector.to_string(),
                    ))?;
                gallery_visuals.push(GalleryVisual::Video(video_src));
            } else {
                let modal_slideshow_el =
                    self.tab.find_element(".modal-package .slideshow-component");
                if modal_slideshow_el.is_err() {
                    let anchor_selector = Selector::new("#main-wrapper > .main-content .gig-page > .main .gallery-slideshow .current .slide figure".to_owned());
                    self.tab
                        .find_element(anchor_selector.as_ref())
                        .map_err(Self::map_err_fn(anchor_selector.to_owned()))?
                        .click()
                        .map_err(Self::map_err_fn(anchor_selector))?;
                    let modal_slide_show_selector =
                        Selector::new(".modal-package .slideshow-component".to_owned());
                    self.tab
                        .wait_for_element(modal_slide_show_selector.as_ref())
                        .map_err(Self::map_err_fn(modal_slide_show_selector))?;
                }
                gallery_visuals.clear();

                let modal_slides_selector = Selector::new(
                    ".modal-package .slideshow-component .slideshow-slide".to_owned(),
                );
                let modal_slides_count = self
                    .tab
                    .find_elements(modal_slides_selector.as_ref())
                    .map_err(Self::map_err_fn(modal_slides_selector))?
                    .len();
                // go to the first slide
                let first_slide_selector = Selector::new(
                    ".modal-package .slideshow-component .slideshow-slide:nth-child(1).current"
                        .to_owned(),
                );
                println!("Navigating to the first slide.");
                while self
                    .tab
                    .find_element(first_slide_selector.as_ref())
                    .is_err()
                {
                    let modal_prev_button_selector =
                        Selector::new(".modal-package .modal-nav-prev".to_owned());
                    self.tab
                        .find_element(modal_prev_button_selector.as_ref())
                        .map_err(Self::map_err_fn(modal_prev_button_selector.to_owned()))?
                        .click()
                        .map_err(Self::map_err_fn(modal_prev_button_selector))?;
                    sleep(Duration::from_millis(100));
                }
                for index in 1..=modal_slides_count {
                    let current_slide_selector = Selector::new(format!(
                        ".modal-package .slideshow-component .slideshow-slide:nth-child({index}).current .slide"
                    ));
                    let current_slide_el =
                        self.tab
                            .find_element(current_slide_selector.as_ref())
                            .map_err(Self::map_err_fn(current_slide_selector.to_owned()))?;
                    let class_name = current_slide_el
                        .get_attribute_value("class")
                        .map_err(Self::map_err_fn(current_slide_selector.to_owned()))?
                        .ok_or(GigVisualsError::ClassAttributeNotFound(
                            current_slide_selector.to_string(),
                        ))?;
                    if class_name.contains("slide-video") {
                        let button_selector = "button";

                        current_slide_el
                            .find_element(button_selector)
                            .map_err(Self::map_err_fn(
                                current_slide_selector.append(button_selector),
                            ))?
                            .click()
                            .map_err(Self::map_err_fn(
                                current_slide_selector.append(button_selector),
                            ))?;
                        let video_el_selector = Selector::new(format!(
                            ".modal-package .slideshow-component .slideshow-slide:nth-child({index}).current .slide video"
                        ));
                        let video_el = self
                            .tab
                            .wait_for_element(video_el_selector.as_ref())
                            .map_err(Self::map_err_fn(video_el_selector.to_owned()))?;
                        let video_src = video_el
                            .get_attribute_value("src")
                            .map_err(Self::map_err_fn(video_el_selector.to_owned()))?
                            .ok_or(GigVisualsError::SrcAttributeNotFound(
                                video_el_selector.to_string(),
                            ))?;
                        gallery_visuals.push(GalleryVisual::Video(video_src));
                    } else {
                        let slide_img_selector = "img";
                        let current_slide_image_el =
                            current_slide_el.find_element(slide_img_selector).map_err(
                                Self::map_err_fn(current_slide_selector.append(slide_img_selector)),
                            )?;
                        let image_src = current_slide_image_el
                            .get_attribute_value("src")
                            .map_err(Self::map_err_fn(
                                current_slide_selector.append(slide_img_selector),
                            ))?
                            .ok_or(GigVisualsError::SrcAttributeNotFound(
                                current_slide_selector
                                    .append(slide_img_selector)
                                    .to_string(),
                            ))?;
                        gallery_visuals.push(GalleryVisual::Image(image_src));
                    }

                    println!("Navigating to the next slide.");
                    let modal_next_button_selector =
                        Selector::new(".modal-package .modal-nav-next".to_owned());
                    self.tab
                        .find_element(modal_next_button_selector.as_ref())
                        .map_err(Self::map_err_fn(modal_next_button_selector.to_owned()))?
                        .click()
                        .map_err(Self::map_err_fn(modal_next_button_selector))?;
                    sleep(Duration::from_millis(100));
                }
                return Ok(gallery_visuals);
            }
        }
        Ok(gallery_visuals)
    }
}
