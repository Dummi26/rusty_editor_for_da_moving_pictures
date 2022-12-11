use image::{DynamicImage, GenericImageView, GenericImage, Pixel, Rgba};

use crate::{curve::Curve, video_render_settings::VideoRenderSettings, content::{content::Content, image::ImageChanges, input_video::InputVideoChanges}, cli::Clz};

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

    generic_content_data: crate::content::content::GenericContentData,
    pub as_content_changes: VideoChanges,
}
#[derive(Default)]
pub struct VideoChanges {
    pub pos: Option<(Option<Curve>, Option<Curve>, Option<Curve>, Option<Curve>)>,
    pub start: Option<f64>,
    pub length: Option<f64>,
    pub video: Option<VideoTypeChanges>,
    pub wrap: Option<VideoChangesWrapWith>,
    pub replace: Option<VideoChangesReplaceWith>,
}
#[derive(Clone)]
pub enum VideoChangesWrapWith {
    List,
    AspectRatio(Curve, Curve),
    WithEffect,
}
#[derive(Clone)]
pub enum VideoChangesReplaceWith {
    List,
    AspectRatio,
    WithEffect,
    Text,
    Image,
    Raw,
    Ffmpeg,
}
impl Content for Video {
    fn clone_no_caching(&self) -> Self {
        Self::new(self.set_pos.clone(), self.set_start_frame.clone(), self.set_length.clone(), self.video.clone_no_caching())
    }
    
    fn children(&self) -> Vec<&Self> {
        match &self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter().collect(),
            VideoTypeEnum::AspectRatio(v, _, _) => vec![v.as_ref()],
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_ref()],
            VideoTypeEnum::Text(_) |
            VideoTypeEnum::Image(_) |
            VideoTypeEnum::Raw(_) |
            VideoTypeEnum::Ffmpeg(_) => Vec::new(),
        }
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        match &mut self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter_mut().collect(),
            VideoTypeEnum::AspectRatio(v, _, _) => vec![v.as_mut()],
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_mut()],
            VideoTypeEnum::Text(_) |
            VideoTypeEnum::Image(_) |
            VideoTypeEnum::Raw(_) |
            VideoTypeEnum::Ffmpeg(_) => Vec::new(),
            
        }
    }
    
    fn has_changes(&self) -> bool {
        self.as_content_changes.pos.is_some()
        | self.as_content_changes.start.is_some()
        | self.as_content_changes.length.is_some()
        | self.as_content_changes.video.is_some()
        | self.as_content_changes.wrap.is_some()
        | self.as_content_changes.replace.is_some()
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
        if let Some(wrap) = self.as_content_changes.wrap.take() {
            match wrap {
                VideoChangesWrapWith::List => {
                    let me = std::mem::replace(&mut self.video, VideoType::new(VideoTypeEnum::List(vec![])));
                    if let VideoTypeEnum::List(l) = &mut self.video.vt { l.push(Video::new_full(me)); }
                }
                VideoChangesWrapWith::AspectRatio(w, h) => {
                    let me = std::mem::replace(&mut self.video, VideoType::new(VideoTypeEnum::List(vec![])));
                    self.video = VideoType::new(VideoTypeEnum::AspectRatio(Box::new(Video::new_full(me)), w, h));
                },
                VideoChangesWrapWith::WithEffect => {
                    let me = std::mem::replace(&mut self.video, VideoType::new(VideoTypeEnum::List(vec![])));
                    self.video = VideoType::new(VideoTypeEnum::WithEffect(
                        Box::new(Video::new_full(me)),
                        crate::effect::Effect::new_from_enum(crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing::new()))
                    ));
                },
            }
        }
        if let Some(replace_with) = self.as_content_changes.replace.take() {
            'replace_with: {
                match (&self.video.vt, &replace_with) {
                    // no change
                    (VideoTypeEnum::List(..), VideoChangesReplaceWith::List)
                    | (VideoTypeEnum::AspectRatio(..), VideoChangesReplaceWith::AspectRatio)
                    | (VideoTypeEnum::WithEffect(..), VideoChangesReplaceWith::WithEffect)
                    | (VideoTypeEnum::Text(..), VideoChangesReplaceWith::Text)
                    | (VideoTypeEnum::Image(..), VideoChangesReplaceWith::Image)
                    | (VideoTypeEnum::Raw(..), VideoChangesReplaceWith::Raw)
                    | (VideoTypeEnum::Ffmpeg(..), VideoChangesReplaceWith::Ffmpeg)
                    => break 'replace_with,
                    _ => (),
                }
                let me = std::mem::replace(&mut self.video.vt, VideoTypeEnum::List(vec![]));
                let new = match (me, replace_with) {
                    // no change
                    (VideoTypeEnum::List(..), VideoChangesReplaceWith::List)
                    | (VideoTypeEnum::AspectRatio(..), VideoChangesReplaceWith::AspectRatio)
                    | (VideoTypeEnum::WithEffect(..), VideoChangesReplaceWith::WithEffect)
                    | (VideoTypeEnum::Text(..), VideoChangesReplaceWith::Text)
                    | (VideoTypeEnum::Image(..), VideoChangesReplaceWith::Image)
                    | (VideoTypeEnum::Raw(..), VideoChangesReplaceWith::Raw)
                    | (VideoTypeEnum::Ffmpeg(..), VideoChangesReplaceWith::Ffmpeg)
                    => unreachable!(), // because of the break 'replace_with above
                    // raw (no change because things will probably break/crash if we try to do pretty much anyting)
                    (me, VideoChangesReplaceWith::Raw) => { println!("video from directory full of frames is implemented quite badly and might cause issues here. sorry."); me }, // TODO!
                    // from/to list
                    (VideoTypeEnum::List(mut v), VideoChangesReplaceWith::AspectRatio) => {
                        if v.is_empty() {
                            VideoTypeEnum::AspectRatio(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))), Curve::Constant(1.0), Curve::Constant(1.0))
                        } else { // put the first element of the list into the aspect ratio (use WRAP instead to preserve all elements)
                            VideoTypeEnum::AspectRatio(Box::new(v.swap_remove(0)), Curve::Constant(1.0), Curve::Constant(1.0))
                        }
                    },
                    (VideoTypeEnum::List(mut v), VideoChangesReplaceWith::WithEffect) => {
                        VideoTypeEnum::WithEffect(Box::new(if v.is_empty() {
                            Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))
                        } else { v.swap_remove(0) // put the first element of the list into the aspect ratio (use WRAP instead to preserve all elements)
                        }), crate::effect::Effect::new_from_enum(crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing::new())))
                    },
                    (VideoTypeEnum::AspectRatio(v, _, _) | VideoTypeEnum::WithEffect(v, _), VideoChangesReplaceWith::List) => VideoTypeEnum::List(vec![*v]),
                    // both things have one child
                    (VideoTypeEnum::AspectRatio(v, _, _), VideoChangesReplaceWith::WithEffect) => VideoTypeEnum::WithEffect(v, crate::effect::Effect::new_from_enum(crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing::new()))),
                    (VideoTypeEnum::WithEffect(v, _), VideoChangesReplaceWith::AspectRatio) => VideoTypeEnum::AspectRatio(v, Curve::Constant(1.0), Curve::Constant(1.0)),
                    // all the things with a path
                    (VideoTypeEnum::Image(v), VideoChangesReplaceWith::Ffmpeg) => VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(v.path().clone())),
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Image) => VideoTypeEnum::Image(crate::content::image::Image::new(v.get_dir().clone())),
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Ffmpeg) => VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(v.get_dir().clone())),
                    (VideoTypeEnum::Ffmpeg(v), VideoChangesReplaceWith::Image) => VideoTypeEnum::Image(crate::content::image::Image::new(v.path().clone())),
                    // to text (where a string representation makes sense) and back
                    (VideoTypeEnum::Image(v), VideoChangesReplaceWith::Text) => VideoTypeEnum::Text(crate::content::text::Text::new(crate::content::text::TextType::Static(v.path().to_string_lossy().to_string()))),
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Text) => VideoTypeEnum::Text(crate::content::text::Text::new(crate::content::text::TextType::Static(v.get_dir().to_string_lossy().to_string()))),
                    (VideoTypeEnum::Ffmpeg(v), VideoChangesReplaceWith::Text) => VideoTypeEnum::Text(crate::content::text::Text::new(crate::content::text::TextType::Static(v.path().to_string_lossy().to_string()))),
                    (VideoTypeEnum::Text(t), into) => {
                        let text = match t.text() {
                            crate::content::text::TextType::Static(t) => t.clone(),
                            crate::content::text::TextType::Program(p) => p.path.to_string_lossy().to_string(),
                        };
                        match into {
                            VideoChangesReplaceWith::List => VideoTypeEnum::List(vec![]),
                            VideoChangesReplaceWith::AspectRatio => VideoTypeEnum::AspectRatio(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))), Curve::Constant(1.0), Curve::Constant(1.0)),
                            VideoChangesReplaceWith::WithEffect => VideoTypeEnum::WithEffect(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))), crate::effect::Effect::new_from_enum(crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing::new()))),
                            VideoChangesReplaceWith::Text => unreachable!(),
                            VideoChangesReplaceWith::Image => VideoTypeEnum::Image(crate::content::image::Image::new(text.into())),
                            VideoChangesReplaceWith::Raw => unreachable!(),
                            VideoChangesReplaceWith::Ffmpeg => VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(text.into())),
                        }
                    },
                    // don't use any information of the old one
                    (_, VideoChangesReplaceWith::List) => VideoTypeEnum::List(vec![]),
                    (_, VideoChangesReplaceWith::AspectRatio) => VideoTypeEnum::AspectRatio(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))), Curve::Constant(1.0), Curve::Constant(1.0)),
                    (_, VideoChangesReplaceWith::WithEffect) => VideoTypeEnum::WithEffect(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(vec![])))), crate::effect::Effect::new_from_enum(crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing::new()))),
                    (_, VideoChangesReplaceWith::Text) => VideoTypeEnum::Text(crate::content::text::Text::new(crate::content::text::TextType::Static("[some text]".to_string()))),
                    (_, VideoChangesReplaceWith::Image) => VideoTypeEnum::Image(crate::content::image::Image::new("[image]".into())),
                    (_, VideoChangesReplaceWith::Ffmpeg) => VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new("[video]".into())),
                };
                self.video.vt = new;
                out = true;
            }
        }
        out && !err

    }

    fn generic_content_data(&mut self) -> &mut crate::content::content::GenericContentData { &mut self.generic_content_data }
}
impl Video {
    pub fn new_full(video: VideoType) -> Self {
        Self {
            set_pos: Pos { align: PosAlign::Center, x: Curve::Constant(0.5), y: Curve::Constant(0.5), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: 0.0,
            set_length: 1.0,
            video,
            transparency_adjustments: TransparencyAdjustments::None,
            generic_content_data: crate::content::content::GenericContentData::default(),
            as_content_changes: VideoChanges::default(),
        }
    }
    pub fn new_full_size(start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: Pos { align: PosAlign::Center, x: Curve::Constant(0.5), y: Curve::Constant(0.5), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: start_frame,
            set_length: length,
            video,
            transparency_adjustments: TransparencyAdjustments::None,
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
            generic_content_data: crate::content::content::GenericContentData::default(),
            as_content_changes: VideoChanges::default(),
        }
    }

    pub fn prep_draw(&self, outer_progress: f64) -> Option<PrepDrawData> {
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
    pub fn draw(&mut self, img: &mut DynamicImage, prep_data: PrepDrawData, render_settings: &mut VideoRenderSettings) {
        //
        let pos_xy = prep_data.position.top_left_xy();
        let pos = Pos { align: PosAlign::TopLeft, x: (pos_xy.0 * img.width() as f64).round() as i32, y: (pos_xy.1 * img.height() as f64).round() as i32, w: (prep_data.position.w * img.width() as f64).round() as u32, h: (prep_data.position.h * img.height() as f64).round() as u32 };
        let prev_frame = render_settings.this_frame.my_size;
        render_settings.this_frame.my_size.0 *= prep_data.position.w;
        render_settings.this_frame.my_size.1 *= prep_data.position.h;
        //
        self.draw2(img, prep_data, pos, render_settings);
        // reset
        render_settings.this_frame.my_size = prev_frame;
    }

    fn draw2(&mut self, image: &mut DynamicImage, prep_data: PrepDrawData, pos: Pos<i32, u32>, render_settings: &mut VideoRenderSettings) {

        let progress = prep_data.progress;

        {
            // Rendering
            let img = self.create_rendered_image(&pos, progress, render_settings);
            // Drawing
            draw_to_canvas(image, &pos, &img, prep_data.transparency_adjustments.convert(&|c| c.get_value(progress) as f32));
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
                        (oa as f32 + (255.0 - oa as f32) * alpha).round() as u8, // TODO: verify that this actually makes sense
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
    fn create_rendered_image(&mut self, pos: &Pos<i32, u32>, progress: f64, render_settings: &mut VideoRenderSettings) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(pos.w, pos.h);
        self.draw3(progress, render_settings, &mut img); // draw onto this image
        img
    }


    /// crop: new min x and y (0 if not cropped), new right and bottom bounds (original width and height if not cropped or only cropped on left and/or top, otherwise width or height minus cropped pixels)
    fn draw3(&mut self, progress: f64, render_settings: &mut VideoRenderSettings, image: &mut DynamicImage) {

        match &mut self.video.vt {



            VideoTypeEnum::Raw(raw_img) => {
                //println!("Drawing RAW");
                let img = raw_img.get_frame_fast(progress, render_settings.max_distance_when_retrieving_closest_frame);
                if let Some(img) = img {
                    img.draw(image, render_settings.image_scaling_filter_type);
                };
            },



            VideoTypeEnum::Ffmpeg(vid) => {
                vid.load_img_force_factor(progress);
                vid.draw(image, render_settings.image_scaling_filter_type)
            },



            VideoTypeEnum::Image(img) => {
                img.draw(image, render_settings.image_scaling_filter_type)
            },



            VideoTypeEnum::Text(txt) => {
                txt.draw(image, progress, self.set_pos.align.convert(|c| c.get_value(progress)).get_anchor(0.0, 0.5, 1.0))
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



            VideoTypeEnum::AspectRatio(vid, width, height) => {
                if let Some(mut prep_draw) = vid.prep_draw(progress) {
                    let my_aspect_ratio = render_settings.this_frame.out_vid_aspect_ratio
                        * (render_settings.this_frame.my_size.0/* * prep_draw.position.w*/) / (render_settings.this_frame.my_size.1/* * prep_draw.position.h*/);
                    // println!("My AR: {}", my_aspect_ratio);
                    let (width, height) = (width.get_value(progress), height.get_value(progress));
                    if height != 0.0 {
                        let desired_aspect_ratio = width / height;
                        let (anchor_x, anchor_y) = prep_draw.position.align.get_anchor(0.0, 0.5, 1.0);
                        if my_aspect_ratio > desired_aspect_ratio { // too wide
                            let w = desired_aspect_ratio / my_aspect_ratio; // < 1
                            let x = (1.0 - w) * anchor_x;
                            prep_draw.position.w *= w;
                            prep_draw.position.x = x + prep_draw.position.x * w;
                        } else if my_aspect_ratio < desired_aspect_ratio { // too high
                            let h = my_aspect_ratio / desired_aspect_ratio; // < 1
                            let y = (1.0 - h) * anchor_y;
                            prep_draw.position.h *= h;
                            prep_draw.position.y = y + prep_draw.position.y * h;
                        }
                    }
                    vid.draw(image, prep_draw, render_settings);
                };
            },



        };
    }




    /// Assumes align is set to TopLeft!
    fn get_inner_pos(pos_outer: &Pos<i32, u32>, pos_inner: &Pos<f64, f64>) -> Pos<i32, u32> {
        Pos {
            x: pos_outer.x + (pos_outer.w as f64 * pos_inner.x).round() as i32,
            y: pos_outer.x + (pos_outer.h as f64 * pos_inner.y).round() as i32,
            w: (pos_outer.w as f64 * pos_inner.w).round() as u32,
            h: (pos_outer.h as f64 * pos_inner.h).round() as u32,
            align: PosAlign::TopLeft,
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
    AspectRatio(Box<Video>, Curve, Curve),
    WithEffect(Box<Video>, crate::effect::Effect),
    Text(crate::content::text::Text),
    Image(crate::content::image::Image),
    Raw(crate::content::input_video::InputVideo),
    Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid),
}
pub enum VideoTypeChanges {
    List(Vec<VideoTypeChanges_List>),
    AspectRatio(Option<Box<VideoChanges>>, Option<Curve>, Option<Curve>),
    WithEffect(Option<Box<VideoChanges>>, Option<crate::effect::Effect>),
    Text(crate::content::text::TextChanges),
    Image(ImageChanges),
    Raw(InputVideoChanges),
    Ffmpeg(super::content::ffmpeg_vid::FfmpegVidChanges),
    ChangeType(VideoTypeEnum),
}

#[allow(non_camel_case_types)]
pub enum VideoTypeChanges_List {
    Swap(usize, usize),
    Move(usize, usize),
    Insert(usize, Video),
    Push(Video),
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
            VideoTypeEnum::AspectRatio(v, width, height) => VideoTypeEnum::AspectRatio(Box::new(v.clone_no_caching()), width.clone(), height.clone()),
            VideoTypeEnum::WithEffect(v, e) => VideoTypeEnum::WithEffect(Box::new(v.clone_no_caching()), e.clone_no_caching()),
            VideoTypeEnum::Text(t) => VideoTypeEnum::Text(t.clone_no_caching()),
            VideoTypeEnum::Image(img) => VideoTypeEnum::Image(img.clone_no_caching()),
            VideoTypeEnum::Raw(v) => VideoTypeEnum::Raw(v.clone_no_caching()),
            VideoTypeEnum::Ffmpeg(v) => VideoTypeEnum::Ffmpeg(v.clone_no_caching()),
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
                            VideoTypeChanges_List::Push(new_val) => { vt.push(new_val); },
                            VideoTypeChanges_List::Change(index, changes) => { let vid = vt.get_mut(index).unwrap(); vid.apply_changes(); vid.as_content_changes = changes; vid.apply_changes(); },
                            VideoTypeChanges_List::Replace(index, new_val) => *vt.get_mut(index).unwrap() = new_val,
                            VideoTypeChanges_List::Remove(index) => { vt.remove(index); },
                        };
                    };
                    true
                },
                (VideoTypeChanges::AspectRatio(vid_changes, width, height), VideoTypeEnum::AspectRatio(v, w, h)) => {
                    let mut out = true;
                    if let Some(changes) = vid_changes {
                        v.as_content_changes = *changes;
                        if !v.apply_changes() { out = false; };
                    }
                    if let Some(width) = width {
                        *w = width;
                    }
                    if let Some(height) = height {
                        *h = height;
                    }
                    out
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
                        VideoTypeChanges::AspectRatio(_, _, _) => "AspectRatio",
                        VideoTypeChanges::WithEffect(_, _) => "WithEffect",
                        VideoTypeChanges::Text(_) => "Text",
                        VideoTypeChanges::Image(_) => "Image",
                        VideoTypeChanges::Raw(_) => "Video",
                        VideoTypeChanges::Ffmpeg(_) => "ffmpeg",
                    }), Clz::error_details(" to data of type "), Clz::error_cause(match data {
                        VideoTypeEnum::List(_) => "List",
                        VideoTypeEnum::AspectRatio(_, _, _) => "AspectRatio",
                        VideoTypeEnum::WithEffect(_, _) => "WithEffect",
                        VideoTypeEnum::Text(_) => "Text",
                        VideoTypeEnum::Image(_) => "Image",
                        VideoTypeEnum::Raw(_) => "Video",
                        VideoTypeEnum::Ffmpeg(_) => "ffmpeg",
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
    pub align: PosAlign<T>,
}
#[derive(Clone, Copy)]
pub enum PosAlign<T> {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
    Custom(T, T),
}
impl<T> PosAlign<T> {
    fn convert<U, F>(&self, converter: F) -> PosAlign<U> where F: Fn(&T) -> U {
        match self {
            PosAlign::TopLeft => PosAlign::TopLeft,
            PosAlign::Top => PosAlign::Top,
            PosAlign::TopRight => PosAlign::TopRight,
            PosAlign::Left => PosAlign::Left,
            PosAlign::Center => PosAlign::Center,
            PosAlign::Right => PosAlign::Right,
            PosAlign::BottomLeft => PosAlign::BottomLeft,
            PosAlign::Bottom => PosAlign::Bottom,
            PosAlign::BottomRight => PosAlign::BottomRight,
            PosAlign::Custom(a, b) => PosAlign::Custom(converter(a), converter(b)),
        }
    }
    /// a is 0, b is 1/2, c is 1
    fn get_anchor(&self, a: T, b: T, c: T) -> (T, T) where T: Clone {
        match self {
            PosAlign::TopLeft => (a.clone(), a),
            PosAlign::Top => (b, a),
            PosAlign::TopRight => (c, a),
            PosAlign::Left => (a, b),
            PosAlign::Center => (b.clone(), b),
            PosAlign::Right => (c, b),
            PosAlign::BottomLeft => (a, c),
            PosAlign::Bottom => (b, c),
            PosAlign::BottomRight => (c.clone(), c),
            PosAlign::Custom(x, y) => (x.clone(), y.clone()),
        }
    }
}

impl<T, U> Pos<T, U> where T: Clone, U: Clone {
    /// Converts all values from T or U to A, including the ones in self.align
    pub fn convert<A, F>(&self, converter: &F) -> Pos<A, A>
    where F: Fn(&T) -> A + Fn(&U) -> A, A: Clone {
        self.convert_sep(converter, converter, |c| c.convert(converter))
    }

    pub fn convert_sep<A, B, F, G, H>(&self, converter_pos: F, converter_dimen: G, converter_align: H) -> Pos<A, B>
    where F: Fn(&T) -> A, G: Fn(&U) -> B, H: Fn(&PosAlign<T>) -> PosAlign<A>, A: Clone, B: Clone {
        Pos {
            x: converter_pos(&self.x),
            y: converter_pos(&self.y),
            w: converter_dimen(&self.w),
            h: converter_dimen(&self.h),
            align: converter_align(&self.align),
        }
    }
}

impl Pos<f64, f64> {
    pub fn top_left_xy(&self) -> (f64, f64) {
        (match self.align {
            PosAlign::TopLeft | PosAlign::Left | PosAlign::BottomLeft => self.x, // left
            PosAlign::Top | PosAlign::Center | PosAlign::Bottom => self.x - self.w / 2.0, // mid
            PosAlign::TopRight | PosAlign::Right | PosAlign::BottomRight => self.x - self.w, // right
            PosAlign::Custom(v, _) => self.x - v * self.w,
        }, match self.align {
            PosAlign::TopLeft | PosAlign::Top | PosAlign::TopRight => self.y, // top
            PosAlign::Left | PosAlign::Center | PosAlign::Right => self.y - self.h / 2.0, // mid
            PosAlign::BottomLeft | PosAlign::Bottom | PosAlign::BottomRight => self.y - self.h, // bottom
            PosAlign::Custom(_, v) => self.y - v * self.h,
        })
    }
}