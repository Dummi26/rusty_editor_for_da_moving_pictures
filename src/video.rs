use std::{rc::Rc, sync::RwLock, slice::{IterMut, Iter}};

use image::{DynamicImage, GenericImageView, GenericImage, Pixel, Rgba};

use crate::{curve::Curve, video_cached_frames::VideoCachedFrames, video_render_settings::VideoRenderSettings, content::{content::Content, image::ImageChanges, input_video::InputVideoChanges}, cli::Clz};

pub struct Video {
    // - - The video's data (what is to be saved to the project file) - -
    // set: Settings
    pub set_pos: Pos<Curve, Curve>,
    pub set_start_frame: f64,
    pub set_length: f64,
    /// The video
    pub video: VideoType,
    /// Post-Processing, effectively
    pub transparency_adjustments: TransparencyAdjustments<Curve>,
    // - -     -     - -
    // done: The values that are set after drawing
    /// Due to caching, the rendered image might not be exactly the desired one. If this is the case, this value will differ from the progress used by draw() etc.
    pub done_actual_progress: f64,

    /// Caching for performance reasons (at the cost of memory usage)
    pub last_draw: VideoCachedFrames,
    generic_content_data: crate::content::content::GenericContentData,
    pub as_content_changes: VideoChanges,
}
#[derive(Default)]
pub struct VideoChanges {
    pub pos: Option<(Option<Curve>, Option<Curve>, Option<Curve>, Option<Curve>)>,
    pub start: Option<f64>,
    pub length: Option<f64>,
    pub video: Option<VideoTypeChanges>,
}
impl Content for Video {
    fn clone_no_caching(&self) -> Self {
        Self::new(self.set_pos.clone(), self.set_start_frame.clone(), self.set_length.clone(), self.video.clone_no_caching())
    }
    
    fn children(&self) -> Vec<&Self> {
        match &self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter().collect(),
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_ref()],
            VideoTypeEnum::Image(_) |
            VideoTypeEnum::Raw(_) => Vec::new(),
        }
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        match &mut self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter_mut().collect(),
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_mut()],
            VideoTypeEnum::Image(_) |
            VideoTypeEnum::Raw(_) => Vec::new(),
        }
    }
    
    fn has_changes(&self) -> bool {
        self.as_content_changes.pos.is_some() | self.as_content_changes.start.is_some() | self.as_content_changes.length.is_some() | self.as_content_changes.video.is_some()
    }
    fn apply_changes(&mut self) -> bool {
        let mut out = false;
        let mut err = false;
        if let Some(mut pos) = self.as_content_changes.pos.take() {
            if let Some(x) = pos.0.take() {
                self.set_pos.x = x;
            };
            if let Some(y) = pos.1.take() {
                self.set_pos.y = y;
            };
            if let Some(w) = pos.2.take() {
                self.set_pos.w = w;
            };
            if let Some(h) = pos.3.take() {
                self.set_pos.h = h;
            };
            out = true;
        };
        if let Some(start) = self.as_content_changes.start.take() {
            self.set_start_frame = start;
            out = true;
        };
        if let Some(length) = self.as_content_changes.length.take() {
            self.set_length = length;
            out = true;
        };
        if let Some(video) = self.as_content_changes.video.take() {
            self.video.apply_changes();
            self.video.changes = Some(video);
            if !self.video.apply_changes() { err = true; };
            out = true;
        };
        self.last_draw.clear_resolutions();
        out && !err
        
    }
    
    fn generic_content_data(&mut self) -> &mut crate::content::content::GenericContentData { &mut self.generic_content_data }
}
impl Video {
    pub fn new_full(video: VideoType) -> Self {
        Self {
            set_pos: Pos { x: Curve::Constant(0.0), y: Curve::Constant(0.0), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: 0.0,
            set_length: 1.0,
            video,
            transparency_adjustments: TransparencyAdjustments::None,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            generic_content_data: crate::content::content::GenericContentData::default(),
            as_content_changes: VideoChanges::default(),
        }
    }
    pub fn new_full_size(start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: Pos { x: Curve::Constant(0.0), y: Curve::Constant(0.0), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: start_frame,
            set_length: length,
            video,
            transparency_adjustments: TransparencyAdjustments::None,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            generic_content_data: crate::content::content::GenericContentData::default(),
            as_content_changes: VideoChanges::default(),
        }
    }
    pub fn new(pos: Pos<Curve, Curve>, start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: pos,
            set_start_frame: start_frame,
            set_length: length,
            video,
            transparency_adjustments: TransparencyAdjustments::None,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            generic_content_data: crate::content::content::GenericContentData::default(),
            as_content_changes: VideoChanges::default(),
        }
    }

    pub fn prep_draw(&mut self, outer_progress: f64) -> Option<PrepDrawData> {
        // handle outer_progress
        if outer_progress < self.set_start_frame { return None; };
        let frames_since_start = outer_progress - self.set_start_frame;
        if frames_since_start >= self.set_length { return None; };
        if self.set_length <= 0.0 { return None; }
        let progress = frames_since_start / self.set_length;
        //
        Some(PrepDrawData {
            position: self.set_pos.convert(&|c| c.get_value(progress)),
            progress,
            transparency_adjustments: self.transparency_adjustments.clone(),
            _private: (),
        })
    }
}
pub struct PrepDrawData {
    pub progress: f64,
    pub position: Pos<f64, f64>,
    pub transparency_adjustments: TransparencyAdjustments<Curve>,
    /// This prevents construction of this struct
    _private: (),
}
impl Video {
    /// This may only be called after prep_draw (which is why it consumes PrepDrawData).
    /// Between prep_draw and draw, effects can make some changes to the PrepDrawData.
    pub fn draw(&mut self, img: &mut DynamicImage, prep_data: PrepDrawData, render_settings: &VideoRenderSettings) {
        //
        let pos = Pos { x: (prep_data.position.x * img.width() as f64).round() as i32, y: (prep_data.position.y * img.height() as f64).round() as i32, w: (prep_data.position.w * img.width() as f64).round() as u32, h: (prep_data.position.h * img.height() as f64).round() as u32 };
        //
        self.draw2(img, prep_data, pos, render_settings);
    }

    fn draw2(&mut self, image: &mut DynamicImage, prep_data: PrepDrawData, pos: Pos<i32, u32>, render_settings: &VideoRenderSettings) {
        
        let progress = prep_data.progress;

        {
            let cached_frames_of_correct_resolution = if render_settings.allow_retrieval_of_cached_frames != None { self.last_draw.with_resolution(pos.w, pos.h) } else { None };
            match cached_frames_of_correct_resolution {
                Some(cache) => {
                    match cache.cache().lock().unwrap().get_frame(progress) {
                        Some((dist, frame)) => {
                            if dist <= render_settings.allow_retrieval_of_cached_frames.expect("This always exists because for cfocr to be Some, arocf cannot be none.") {
                                self.done_actual_progress = frame.progress;
                                draw_to_canvas(image, &pos, &frame.frame, prep_data.transparency_adjustments.convert(&|c| c.get_value(progress) as f32));
                                return; // drawing from cache, return to prevent rendering
                            };
                        },
                        None => {},
                    };
                },
                None => {},
            };
        };
        {
            // Rendering
            let img = self.create_rendered_image(&pos, progress, render_settings);
            // Drawing
            draw_to_canvas(image, &pos, &img, prep_data.transparency_adjustments.convert(&|c| c.get_value(progress) as f32));
            // Caching
            self.last_draw.add_frame(progress, pos.w, pos.h, img);
        };
        fn draw_to_canvas(image: &mut DynamicImage, pos: &Pos<i32, u32>, img: &DynamicImage, transparency_adjustments: TransparencyAdjustments<f32>) {

            // Draw frame to canvas
    
            let cropped_left = pos.x < 0;
            let cropped_left_by = if cropped_left { -pos.x as u32 } else { 0 };
            let cropped_top = pos.y < 0;
            let cropped_top_by = if cropped_top { -pos.y as u32 } else { 0 };
            //let cropped_right = add(pos.x, pos.w) > image.width();
            //let cropped_bottom = add(pos.y, pos.h) > image.height();
            
            /// Panicks if i is negative and -i > u
            fn add(i: i32, u: u32) -> u32 { if i < 0 { u - (-i) as u32 } else { u + i as u32 } }
    
            for pixel in img.pixels() {
                let (x, y, pixel) = (pixel.0, pixel.1, pixel.2);
                // ensure our x and y coordinates for put_pixel are not negative
                if (cropped_left && x < cropped_left_by) // pixel out on left
                || (cropped_top && y < cropped_top_by) // pixel out on top
                { continue; };
                let (x, y) = (add(pos.x, x), add(pos.y, y));
                if x >= image.width() || y >= image.height() { continue; };
                let px = image.get_pixel(x, y);
                let [nr, ng, nb, na] = pixel.0;
                let [or, og, ob, oa] = px.0;
                let alpha = match transparency_adjustments {
                    TransparencyAdjustments::None => na as f32 / 255.0,
                    TransparencyAdjustments::Force(v) => v,
                    TransparencyAdjustments::Factor(f) => na as f32 * f / 255.0,
                    TransparencyAdjustments::ForceOpaqueIfNotTransparent => if na == 0 { 0.0 } else { 1.0 },
                };
                if alpha == 0.0 {} // nothing
                else if alpha == 1.0 { // opaque
                    image.put_pixel(x, y, *Rgba::from_slice(&[nr, ng, nb, 255]));
                } else { // transparency
                    let a2 = 1.0 - alpha;
                    image.put_pixel(x, y, *Rgba::from_slice(&[
                        (or as f32 * a2 + nr as f32 * alpha).round() as u8,
                        (og as f32 * a2 + ng as f32 * alpha).round() as u8,
                        (ob as f32 * a2 + nb as f32 * alpha).round() as u8,
                        255
                    ]));
                };
            };
        
        }
    }
}

#[derive(Clone)]
pub enum TransparencyAdjustments<T> {
    None,
    Force(T),
    Factor(T),
    ForceOpaqueIfNotTransparent,
}
impl<T> TransparencyAdjustments<T> {
    pub fn convert<F, R>(self, f: &F) -> TransparencyAdjustments<R> where F: Fn(T) -> R {
        match self {
            Self::None => TransparencyAdjustments::None,
            Self::Force(t) => TransparencyAdjustments::Force(f(t)),
            Self::Factor(t) => TransparencyAdjustments::Factor(f(t)),
            Self::ForceOpaqueIfNotTransparent => TransparencyAdjustments::ForceOpaqueIfNotTransparent,
        }
    }
}

impl Video {


    /// This function does not put the image into the cache automatically. To do that, use self.last_draw.add_frame(?, ?, ?, create_rendered_image(...))
    fn create_rendered_image(&mut self, pos: &Pos<i32, u32>, progress: f64, render_settings: &VideoRenderSettings) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(pos.w, pos.h);
        self.draw3(progress, render_settings, &mut img); // draw onto this image
        self.done_actual_progress = progress;
        img
    }


    /// crop: new min x and y (0 if not cropped), new right and bottom bounds (original width and height if not cropped or only cropped on left and/or top, otherwise width or height minus cropped pixels)
    fn draw3(&mut self, progress: f64, render_settings: &VideoRenderSettings, image: &mut DynamicImage) {

        match &mut self.video.vt {



            VideoTypeEnum::Raw(raw_img) => {
                //println!("Drawing RAW");
                let img = raw_img.get_frame_fast(progress, render_settings.max_distance_when_retrieving_closest_frame);
                if let Some(img) = img {
                    img.draw(image, render_settings.image_scaling_filter_type);
                };
            },
            
            
            
            VideoTypeEnum::Image(img) => {
                img.draw(image, render_settings.image_scaling_filter_type);
            },



            VideoTypeEnum::WithEffect(vid, effect) => {
                effect.process_image(progress, vid, image, render_settings);
            },



            VideoTypeEnum::List(others) => {
                //println!("Drawing LIST");
                for other in others {
                    if let Some(prep_draw) = other.prep_draw(progress) {
                        other.draw(image, prep_draw, render_settings);
                    };
                };
            },



        };
    }





    fn get_inner_pos(pos_outer: &Pos<i32, u32>, pos_inner: &Pos<f64, f64>) -> Pos<i32, u32> {
        Pos {
            x: pos_outer.x + (pos_outer.w as f64 * pos_inner.x).round() as i32,
            y: pos_outer.x + (pos_outer.h as f64 * pos_inner.y).round() as i32,
            w: (pos_outer.w as f64 * pos_inner.w).round() as u32,
            h: (pos_outer.h as f64 * pos_inner.h).round() as u32,
        }
    }
}

pub struct VideoType {
    pub vt: VideoTypeEnum,
    generic_content_data: crate::content::content::GenericContentData,
    changes: Option<VideoTypeChanges>,
} impl VideoType { pub fn new(vt: VideoTypeEnum) -> Self { Self { vt, changes: None, generic_content_data: crate::content::content::GenericContentData::default(), }
} }
pub enum VideoTypeEnum {
    List(Vec<Video>),
    WithEffect(Box<Video>, crate::effect::Effect),
    Image(crate::content::image::Image),
    Raw(crate::content::input_video::InputVideo),
}
pub enum VideoTypeChanges {
    List(Vec<VideoTypeChanges_List>),
    WithEffect(Option<Box<VideoChanges>>, Option<crate::effect::Effect>),
    Image(ImageChanges),
    Raw(InputVideoChanges),
    ChangeType(VideoTypeEnum),
}

#[allow(non_camel_case_types)]
pub enum VideoTypeChanges_List {
    Swap(usize, usize),
    Move(usize, usize),
    Insert(usize, Video),
    Change(usize, VideoChanges),
    Replace(usize, Video),
    Remove(usize),
}

impl Content for VideoType {
    fn clone_no_caching(&self) -> Self {
        VideoType::new(match &self.vt {
            VideoTypeEnum::List(vec) => VideoTypeEnum::List({
                let mut nvec = Vec::with_capacity(vec.len());
                for v in vec {
                    nvec.push(v.clone_no_caching());
                };
                nvec
            }),
            VideoTypeEnum::WithEffect(v, e) => VideoTypeEnum::WithEffect(Box::new(v.clone_no_caching()), e.clone_no_caching()),
            VideoTypeEnum::Image(img) => VideoTypeEnum::Image(img.clone_no_caching()),
            VideoTypeEnum::Raw(v) => VideoTypeEnum::Raw(v.clone_no_caching()),
        })
    }
    
    fn children(&self) -> Vec<&Self> {
        Vec::new()
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        Vec::new()
    }

    fn has_changes(&self) -> bool {
        self.changes.is_some()
    }
    fn apply_changes(&mut self) -> bool {
        if let Some(changes) = self.changes.take() {
            match (changes, &mut self.vt) {
                (VideoTypeChanges::List(changes), VideoTypeEnum::List(vt)) => {
                    for change in changes {
                        match change {
                            VideoTypeChanges_List::Swap(a, b) => { vt.swap(a, b); },
                            VideoTypeChanges_List::Move(a, b) => { let v = vt.remove(b); vt.insert(a, v); }
                            VideoTypeChanges_List::Insert(index, new_val) => { vt.insert(index, new_val); },
                            VideoTypeChanges_List::Change(index, changes) => { let vid = vt.get_mut(index).unwrap(); vid.apply_changes(); vid.as_content_changes = changes; vid.apply_changes(); },
                            VideoTypeChanges_List::Replace(index, new_val) => *vt.get_mut(index).unwrap() = new_val,
                            VideoTypeChanges_List::Remove(index) => { vt.remove(index); },
                        };
                    };
                    true
                },
                (VideoTypeChanges::WithEffect(vid_changes, eff_new), VideoTypeEnum::WithEffect(vid, eff)) => {
                    let mut out = true;
                    if let Some(changes) = vid_changes {
                        vid.as_content_changes = *changes;
                        if !vid.apply_changes() { out = false; };
                    };
                    if let Some(eff_new) = eff_new {
                        *eff = eff_new;
                    };
                    out
                },
                (VideoTypeChanges::Image(img_changes), VideoTypeEnum::Image(img)) => {
                    img.as_content_changes = img_changes;
                    img.apply_changes()
                },
                (VideoTypeChanges::Raw(changes), VideoTypeEnum::Raw(vid)) => {
                    vid.as_content_changes = changes;
                    vid.apply_changes()
                },
                (VideoTypeChanges::ChangeType(new), _) => {
                    self.vt = new;
                    true
                },
                (changes, data) => panic!("\n{}\n    {}{}{}{}{}\n",
                    Clz::error_info("Attempted to apply VideoTypeChanges, but found different types:"),
                    Clz::error_details("Tried to apply changes of type "), Clz::error_cause(match changes {
                        VideoTypeChanges::ChangeType(_) => "[?]",
                        VideoTypeChanges::List(_) => "List",
                        VideoTypeChanges::WithEffect(_, _) => "WithEffect",
                        VideoTypeChanges::Image(_) => "Image",
                        VideoTypeChanges::Raw(_) => "Video",
                    }), Clz::error_details(" to data of type "), Clz::error_cause(match data {
                        VideoTypeEnum::List(_) => "List",
                        VideoTypeEnum::WithEffect(_, _) => "WithEffect",
                        VideoTypeEnum::Image(_) => "Image",
                        VideoTypeEnum::Raw(_) => "Video",
                    }), Clz::error_details(".")
                ),
            }
        } else { true }
    }
    
    fn generic_content_data(&mut self) -> &mut crate::content::content::GenericContentData { &mut self.generic_content_data }
}

#[derive(Clone)]
pub struct Pos<T, U> where T: Sized + Clone, U: Sized + Clone {
    pub x: T,
    pub y: T,
    pub w: U,
    pub h: U,
}
impl<T, U> Pos<T, U> where T: Clone, U: Clone {
    pub fn convert<A, F>(&self, converter: &F) -> Pos<A, A> where F: Fn(&T) -> A + Fn(&U) -> A, A: Clone {
        self.convert_sep(converter, converter)
    }
    pub fn convert_sep<A, B, F, G>(&self, converter1: F, converter2: G) -> Pos<A, B> where F: Fn(&T) -> A, G: Fn(&U) -> B, A: Clone, B: Clone {
        Pos {
            x: converter1(&self.x),
            y: converter1(&self.y),
            w: converter2(&self.w),
            h: converter2(&self.h),
        }
    }
}