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
    pub fn process_image(&mut self, progress: f64, vid: &mut Video, img: &mut DynamicImage, render_settings: &VideoRenderSettings) {
        self.effect.process_image(progress, vid, img, render_settings);
    }
}

//  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //

pub mod effects {
    use image::{GenericImageView, GenericImage};

    use crate::{video_render_settings::VideoRenderSettings, curve::Curve};

    pub trait Effect: Send {
        /// The default implementation of this should just call prepare_draw and draw on vid.
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings);
    }



    pub struct Nothing {}
    impl Effect for Nothing {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_data) = vid.prep_draw(progress) {
                vid.draw(img, prep_data, render_settings);
            };
        }
    }



    pub struct BlackWhite {
    }
    impl Effect for BlackWhite {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_draw) = vid.prep_draw(progress) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                let [r, g, b, a] = px.0;
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
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings) {
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



    pub struct ChangeSpeed {
        pub time: Curve,
    }
    impl Effect for ChangeSpeed {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(mut prep_data) = vid.prep_draw(progress) {
                prep_data.progress = self.time.get_value(prep_data.progress);
                vid.draw(img, prep_data, render_settings);
            };
        }
    }



    pub struct ColorAdjust {
        pub mode: ColorAdjust_Mode,
    }
    impl Effect for ColorAdjust {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_draw) = vid.prep_draw(progress) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                px.0 = self.mode.get_color(px.0);
            };
        }
    }
    #[allow(non_camel_case_types)]
    pub enum ColorAdjust_Mode {
        Rgba(Curve, Curve, Curve, Curve),
    }
    impl ColorAdjust_Mode {
        pub fn get_color(&self, o: [u8; 4]) -> [u8; 4] {
            match self {
                Self::Rgba(r, g, b, a) => {
                    [
                        (r.get_value(o[0] as f64 / 255.0) * 255.0).max(0.0).min(255.0) as u8,
                        (g.get_value(o[1] as f64 / 255.0) * 255.0).max(0.0).min(255.0) as u8,
                        (b.get_value(o[2] as f64 / 255.0) * 255.0).max(0.0).min(255.0) as u8,
                        (a.get_value(o[3] as f64 / 255.0) * 255.0).max(0.0).min(255.0) as u8,
                    ]
                }
            }
        }
    }



    pub struct Blur {
        pub mode: Blur_Mode,
    }
    pub enum Blur_Mode {
        Square { radius: Curve, },
        Downscale { width: Curve, height: Curve, },
    }
    impl Effect for Blur {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_data) = vid.prep_draw(progress) {
                let mut img2 = super::DynamicImage::new_rgba8(img.width(), img.height());
                vid.draw(&mut img2, prep_data, render_settings);
                match &self.mode {
                    Blur_Mode::Square { radius, } => {
                        let radius = radius.get_value(progress).round() as u32;
                        for y in 0..img.height() {
                            for x in 0..img.width() {
                                img.put_pixel(x, y, {
                                    let left = x.saturating_sub(radius);
                                    let right = (x + radius).min(img.width()-1);
                                    let top = y.saturating_sub(radius);
                                    let bottom = (y + radius).min(img.height()-1);
                                    let width = 1 + right - left;
                                    let height = 1 + bottom - top;
                                    let count = width * height;
                                    let mut r = 0.0f32;
                                    let mut g = 0.0f32;
                                    let mut b = 0.0f32;
                                    //let mut a = 255;
                                    for y2 in top..=bottom {
                                        for x2 in left..=right {
                                            let [r2, g2, b2, _] = img2.get_pixel(x2, y2).0;
                                            r += r2 as f32 / count as f32;
                                            g += g2 as f32 / count as f32;
                                            b += b2 as f32 / count as f32;
                                        };
                                    };
                                    *image::Pixel::from_slice(&[r.round() as u8, g.round() as u8, b.round() as u8, 255])
                                });
                            };
                        };
                    },
                    Blur_Mode::Downscale { width, height, } => {
                        *img = img2.resize_exact(
                            (img2.width() as f64 * width.get_value(progress)).round() as u32,
                            (img2.height() as f64 * height.get_value(progress)).round() as u32,
                            image::imageops::FilterType::Nearest,
                        ).resize_exact(img.width(), img.height(), image::imageops::FilterType::Nearest);
                    },
                };
            };
        }
    }



    pub struct ColorKey {
        pub mode: ColorKey_Mode,
    }
    impl Effect for ColorKey {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &VideoRenderSettings) {
            if let Some(prep_draw) = vid.prep_draw(progress) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                px.0 = self.mode.get_color(px.0);
            };
        }
    }
    #[allow(non_camel_case_types)]
    pub enum ColorKey_Mode {
        TransparentIfMatches((u8, u8, u8)),
        TransparentIfRange(((u8, u8), (u8, u8), (u8, u8))),
    }
    impl ColorKey_Mode {
        pub fn get_color(&self, mut o: [u8; 4]) -> [u8; 4] {
            #[allow(non_snake_case)]
            match self {
                Self::TransparentIfMatches((R, G, B)) => if (o[0], o[1], o[2]) == (*R, *G, *B) {
                    o[3] = 0;
                    o
                } else { o },
                Self::TransparentIfRange(((R1, R2), (G1, G2), (B1, B2))) => if *R1 <= o[0] && o[0] <= *R2 && *G1 <= o[1] && o[1] <= *G2 && *B1 <= o[2] && o[2] <= *B2 {
                    o[3] = 0;
                    o
                } else { o },
            }
        }
    }



}