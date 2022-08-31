use image::{DynamicImage};

use crate::{video::Video, video_render_settings::VideoRenderSettings};

pub struct Effect {
    /// The actual effect
    pub effect: Box<dyn effects::Effect>,
}
impl Effect {
    pub fn new<T>(effect: T) -> Self where T: effects::Effect + 'static {
        Self { effect: Box::new(effect), }
    }
    pub fn process_image(&self, progress: f64, vid: &mut Video, img: &mut DynamicImage, render_settings: &VideoRenderSettings) {
        self.effect.process_image(progress, vid, img, render_settings);
    }
}

//  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //

pub mod effects {
    use crate::video_render_settings::VideoRenderSettings;

    pub trait Effect: Send {
        /// The default implementation of this should just call prepare_draw and draw on vid.
        fn process_image(&self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings);
    }

    pub struct Nothing {}
    impl Effect for Nothing {
        fn process_image(&self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_data) = vid.prep_draw(progress) {
                vid.draw(img, prep_data, render_settings);
            };
        }
    }

    pub struct BlackWhite {
    }
    impl Effect for BlackWhite {
        fn process_image(&self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_draw) = vid.prep_draw(progress) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
                let rgb = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                px[0] = rgb;
                px[1] = rgb;
                px[2] = rgb;
            };
        }
    }

    pub struct Shake {
        pub shake_dist_x: f64,
        pub shake_dist_y: f64,
        pub shakes_count_x: f64,
        pub shakes_count_y: f64,
    }
    impl Effect for Shake {
        fn process_image(&self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(mut prep_data) = vid.prep_draw(progress) {
    
                if self.shakes_count_x > 0.0 {
                    prep_data.position.x += self.shake_dist_x * (2.0 * std::f64::consts::PI * progress * self.shakes_count_x).sin();
                };
                if self.shakes_count_y > 0.0 {
                    prep_data.position.y += self.shake_dist_y * (2.0 * std::f64::consts::PI * progress * self.shakes_count_y).sin();
                };
    
                vid.draw(img, prep_data, render_settings);
            };
        }
    }
}