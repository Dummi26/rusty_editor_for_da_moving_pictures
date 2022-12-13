use image::DynamicImage;

use crate::{video::Video, video_render_settings::VideoRenderSettings};

use effects::EffectT;

pub struct Effect {
    /// The actual effect
    pub effect: effects::EffectsEnum,
}
impl Effect {
    pub fn new<T>(effect: T) -> Self where T: effects::EffectT + 'static {
        Self::new_from_enum(effect.as_enum())
    }
    pub fn new_from_enum(effect: effects::EffectsEnum) -> Self {
        Self { effect, }
    }
    pub fn process_image(&mut self, progress: f64, vid: &mut Video, img: &mut DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
        self.effect.process_image(progress, vid, img, render_settings, parent_prep_draw_data);
    }
    pub fn clone_no_caching(&self) -> Self {
        self.effect.clone_no_caching()
    }
}

//  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //  //

pub mod effects {
    use image::{GenericImageView, GenericImage};

    use crate::{video_render_settings::VideoRenderSettings, curve::Curve};

    pub trait EffectT: Send {
        /// The default implementation of this should just call prepare_draw and draw on vid.
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData);
        /// 
        fn clone_no_caching(&self) -> super::Effect;
        
        fn as_enum(self) -> EffectsEnum;
    }
    pub enum EffectsEnum {
        Nothing(Nothing),
        BlackWhite(BlackWhite),
        Rotate(Rotate),
        Shake(Shake),
        ChangeTime(ChangeTime),
        ColorAdjust(ColorAdjust),
        Blur(Blur),
        ColorKey(ColorKey),
    }
    impl EffectT for EffectsEnum {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            match self {
                EffectsEnum::Nothing(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::BlackWhite(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::Rotate(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::Shake(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::ChangeTime(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::ColorAdjust(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::Blur(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
                EffectsEnum::ColorKey(v) => v.process_image(progress, vid, img, render_settings, parent_prep_draw_data),
            }
        }

        fn clone_no_caching(&self) -> super::Effect {
            match self {
                EffectsEnum::Nothing(v) => v.clone_no_caching(),
                EffectsEnum::BlackWhite(v) => v.clone_no_caching(),
                EffectsEnum::Rotate(v) => v.clone_no_caching(),
                EffectsEnum::Shake(v) => v.clone_no_caching(),
                EffectsEnum::ChangeTime(v) => v.clone_no_caching(),
                EffectsEnum::ColorAdjust(v) => v.clone_no_caching(),
                EffectsEnum::Blur(v) => v.clone_no_caching(),
                EffectsEnum::ColorKey(v) => v.clone_no_caching(),
            }
        }

        fn as_enum(self) -> EffectsEnum {
            self
        }
    }



    pub struct Nothing {
    }
    impl Nothing { pub fn new() -> Self {
        Self {  }
    } }
    impl EffectT for Nothing {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_data) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
                vid.draw(img, prep_data, render_settings);
            };
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new()) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::Nothing(self) }
    }



    pub struct BlackWhite {
    }
    impl BlackWhite { pub fn new() -> Self {
        Self {  }
    } }
    impl EffectT for BlackWhite {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_draw) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
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
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new()) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::BlackWhite(self) }
    }



    pub struct Rotate {
        angle: Curve,
        rotation_point: (Curve, Curve),
        /// The rotate mode to be used.
        rotate_mode: Rotate_Mode,
    }
    impl Rotate { pub fn new() -> Self {
        todo!()
    } }
    impl EffectT for Rotate {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_draw) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
                vid.draw(img, prep_draw, render_settings);
            };
            // Rotate the image by modifying the data in img
            todo!("The rotate effect does not yet rotate the image...")
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new()) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::Rotate(self) }
    }
    #[allow(non_camel_case_types)]
    pub enum Rotate_Mode {
        MirrorPoint,
        // MirrorVertical,
        // MirrorHorizontal,
        MirrorAxis,
        RotatePoint,
        /// Just like RotatePoint, except the default angle determines the rotation near the center point while the 'far' curve determines the rotation furthest from the center. The 'out' curve determines how much of each curve should be used. Its input is how far outside the affected pixel is (0.0 for the center pixel, more for ones that are far from the center), while its output determines how important the 'far' curve should be: 0.0 means "use only the 'angle' curve" while 1.0 means "use only the 'far' curve".
        RotatePointSpiral {
            out: Curve,
            far: Curve
        },
        // /// In this rotate mode, the provided function is given the progress (0.0 to 1.0) and the image. It can change the pixels of the provided image however it wants.
        // Custom(Box<fn(f64, &mut super::DynamicImage)>), // this probably shouldnt ever be necessary
    }



    pub struct Shake {
        pub shake_dist_x: f64,
        pub shake_dist_y: f64,
        pub shakes_count_x: f64,
        pub shakes_count_y: f64,
    }
    impl Shake { pub fn new(shake_dist_x: f64, shake_dist_y: f64, shakes_count_x: f64, shakes_count_y: f64) -> Self {
        Self { shake_dist_x, shake_dist_y, shakes_count_x, shakes_count_y, }
    } }
    impl EffectT for Shake {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(mut prep_data) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
    
                if self.shakes_count_x > 0.0 {
                    prep_data.position.x += self.shake_dist_x * (2.0 * std::f64::consts::PI * progress * self.shakes_count_x).sin();
                };
                if self.shakes_count_y > 0.0 {
                    prep_data.position.y += self.shake_dist_y * (2.0 * std::f64::consts::PI * progress * self.shakes_count_y).sin();
                };
    
                vid.draw(img, prep_data, render_settings);
            };
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new(self.shake_dist_x.clone(), self.shake_dist_y.clone(), self.shakes_count_x.clone(), self.shakes_count_y.clone())) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::Shake(self) }
    }



    pub struct ChangeTime {
        pub time: Curve,
    }
    impl ChangeTime { pub fn new(time: Curve) -> Self {
        Self { time, }
    } }
    impl EffectT for ChangeTime {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut image::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_data) = vid.prep_draw(self.time.get_value(progress), Some(parent_prep_draw_data)) {
                vid.draw(img, prep_data, render_settings);
            };
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new(self.time.clone())) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::ChangeTime(self) }
    }



    pub struct ColorAdjust {
        pub mode: ColorAdjust_Mode,
    }
    impl ColorAdjust { pub fn new(mode: ColorAdjust_Mode) -> Self {
        Self { mode, }
    } }
    impl EffectT for ColorAdjust {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_draw) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                px.0 = self.mode.get_color(px.0);
            };
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new(self.mode.clone())) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::ColorAdjust(self) }
    }
    #[allow(non_camel_case_types)]
    #[derive(Clone)]
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
    impl Blur { pub fn new(mode: Blur_Mode) -> Self {
        Self { mode, }
    } }
    #[allow(non_camel_case_types)]
    #[derive(Clone)]
    pub enum Blur_Mode {
        Square { radius: Curve, },
        Downscale { width: Curve, height: Curve, },
    }
    impl EffectT for Blur {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_data) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
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
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new(self.mode.clone())) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::Blur(self) }
    }



    pub struct ColorKey {
        pub mode: ColorKey_Mode,
    }
    impl ColorKey { pub fn new(mode: ColorKey_Mode) -> Self {
        Self { mode, }
    } }
    impl EffectT for ColorKey {
        fn process_image(&mut self, progress: f64, vid: &mut crate::video::Video, img: &mut super::DynamicImage, render_settings: &mut VideoRenderSettings, parent_prep_draw_data: &crate::video::PrepDrawData) {
            if let Some(prep_draw) = vid.prep_draw(progress, Some(parent_prep_draw_data)) {
                vid.draw(img, prep_draw, render_settings);
            };

            for px in img.as_mut_rgba8().unwrap().pixels_mut() {
                px.0 = self.mode.get_color(px.0);
            };
        }
        fn clone_no_caching(&self) -> super::Effect { super::Effect::new(Self::new(self.mode.clone())) }
        fn as_enum(self) -> EffectsEnum { EffectsEnum::ColorKey(self) }
    }
    #[allow(non_camel_case_types)]
    #[derive(Clone)]
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