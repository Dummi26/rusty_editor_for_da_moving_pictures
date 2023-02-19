use image::{DynamicImage, GenericImage, GenericImageView};

use crate::{
    cli::Clz,
    content::{
        content::{Content, GenericContentData},
        image::ImageChanges,
        input_video::InputVideoChanges,
    },
    curve::{Curve, CurveData},
    video_render_settings::VideoRenderSettings,
};

pub struct Video {
    // - - The video's data (what is to be saved to the project file) - -
    // set: Settings
    pub set_pos: Pos<Curve, Curve>,
    pub set_start_frame: f64,
    pub set_length: f64,
    /// The video
    pub video: VideoType,
    /// how to write the pixels onto the underlying surface. If None, inherits from parent.
    pub compositing: Option<CompositingMethod>,
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
        Self::new(
            self.set_pos.clone(),
            self.set_start_frame.clone(),
            self.set_length.clone(),
            self.video.clone_no_caching(),
        )
    }

    fn children(&self) -> Vec<&Self> {
        match &self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter().collect(),
            VideoTypeEnum::AspectRatio(v, _, _) => vec![v.as_ref()],
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_ref()],
            VideoTypeEnum::Text(_)
            | VideoTypeEnum::Image(_)
            | VideoTypeEnum::Raw(_)
            | VideoTypeEnum::Ffmpeg(_) => Vec::new(),
        }
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        match &mut self.video.vt {
            VideoTypeEnum::List(vec) => vec.iter_mut().collect(),
            VideoTypeEnum::AspectRatio(v, _, _) => vec![v.as_mut()],
            VideoTypeEnum::WithEffect(v, _) => vec![v.as_mut()],
            VideoTypeEnum::Text(_)
            | VideoTypeEnum::Image(_)
            | VideoTypeEnum::Raw(_)
            | VideoTypeEnum::Ffmpeg(_) => Vec::new(),
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
            if !self.video.apply_changes() {
                err = true;
            };
            out = true;
        };
        if let Some(wrap) = self.as_content_changes.wrap.take() {
            match wrap {
                VideoChangesWrapWith::List => {
                    let me = std::mem::replace(
                        &mut self.video,
                        VideoType::new(
                            VideoTypeEnum::List(vec![]),
                            self.generic_content_data.clone(),
                        ),
                    );
                    if let VideoTypeEnum::List(l) = &mut self.video.vt {
                        l.push(Video::new_full(me));
                    }
                }
                VideoChangesWrapWith::AspectRatio(w, h) => {
                    let me = std::mem::replace(
                        &mut self.video,
                        VideoType::new(
                            VideoTypeEnum::List(vec![]),
                            self.generic_content_data.clone(),
                        ),
                    );
                    self.video = VideoType::new(
                        VideoTypeEnum::AspectRatio(Box::new(Video::new_full(me)), w, h),
                        self.generic_content_data.clone(),
                    );
                }
                VideoChangesWrapWith::WithEffect => {
                    let me = std::mem::replace(
                        &mut self.video,
                        VideoType::new(
                            VideoTypeEnum::List(vec![]),
                            self.generic_content_data.clone(),
                        ),
                    );
                    self.video = VideoType::new(
                        VideoTypeEnum::WithEffect(
                            Box::new(Video::new_full(me)),
                            crate::effect::Effect::new_from_enum(
                                crate::effect::effects::EffectsEnum::Nothing(
                                    crate::effect::effects::Nothing::new(),
                                ),
                            ),
                        ),
                        self.generic_content_data.clone(),
                    );
                }
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
                    | (VideoTypeEnum::Ffmpeg(..), VideoChangesReplaceWith::Ffmpeg) => {
                        break 'replace_with
                    }
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
                    | (VideoTypeEnum::Ffmpeg(..), VideoChangesReplaceWith::Ffmpeg) => {
                        unreachable!()
                    } // because of the break 'replace_with above
                    // raw (no change because things will probably break/crash if we try to do pretty much anyting)
                    (me, VideoChangesReplaceWith::Raw) => {
                        println!("video from directory full of frames is implemented quite badly and might cause issues here. sorry.");
                        me
                    } // TODO!
                    // from/to list
                    (VideoTypeEnum::List(mut v), VideoChangesReplaceWith::AspectRatio) => {
                        if v.is_empty() {
                            VideoTypeEnum::AspectRatio(
                                Box::new(Video::new_full(VideoType::new(
                                    VideoTypeEnum::List(vec![]),
                                    self.generic_content_data.clone(),
                                ))),
                                CurveData::Constant(1.0).into(),
                                CurveData::Constant(1.0).into(),
                            )
                        } else {
                            // put the first element of the list into the aspect ratio (use WRAP instead to preserve all elements)
                            VideoTypeEnum::AspectRatio(
                                Box::new(v.swap_remove(0)),
                                CurveData::Constant(1.0).into(),
                                CurveData::Constant(1.0).into(),
                            )
                        }
                    }
                    (VideoTypeEnum::List(mut v), VideoChangesReplaceWith::WithEffect) => {
                        VideoTypeEnum::WithEffect(
                            Box::new(if v.is_empty() {
                                Video::new_full(VideoType::new(
                                    VideoTypeEnum::List(vec![]),
                                    self.generic_content_data.clone(),
                                ))
                            } else {
                                v.swap_remove(0) // put the first element of the list into the aspect ratio (use WRAP instead to preserve all elements)
                            }),
                            crate::effect::Effect::new_from_enum(
                                crate::effect::effects::EffectsEnum::Nothing(
                                    crate::effect::effects::Nothing::new(),
                                ),
                            ),
                        )
                    }
                    (
                        VideoTypeEnum::AspectRatio(v, _, _) | VideoTypeEnum::WithEffect(v, _),
                        VideoChangesReplaceWith::List,
                    ) => VideoTypeEnum::List(vec![*v]),
                    // both things have one child
                    (VideoTypeEnum::AspectRatio(v, _, _), VideoChangesReplaceWith::WithEffect) => {
                        VideoTypeEnum::WithEffect(
                            v,
                            crate::effect::Effect::new_from_enum(
                                crate::effect::effects::EffectsEnum::Nothing(
                                    crate::effect::effects::Nothing::new(),
                                ),
                            ),
                        )
                    }
                    (VideoTypeEnum::WithEffect(v, _), VideoChangesReplaceWith::AspectRatio) => {
                        VideoTypeEnum::AspectRatio(
                            v,
                            CurveData::Constant(1.0).into(),
                            CurveData::Constant(1.0).into(),
                        )
                    }
                    // all the things with a path
                    (VideoTypeEnum::Image(v), VideoChangesReplaceWith::Ffmpeg) => {
                        VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(
                            v.path().clone(),
                            self.generic_content_data.reset(),
                        ))
                    }
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Image) => {
                        VideoTypeEnum::Image(crate::content::image::Image::new(
                            v.get_dir().clone(),
                            self.generic_content_data.reset(),
                        ))
                    }
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Ffmpeg) => {
                        VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(
                            v.get_dir().clone(),
                            self.generic_content_data.reset(),
                        ))
                    }
                    (VideoTypeEnum::Ffmpeg(v), VideoChangesReplaceWith::Image) => {
                        VideoTypeEnum::Image(crate::content::image::Image::new(
                            v.path().clone(),
                            self.generic_content_data.reset(),
                        ))
                    }
                    // to text (where a string representation makes sense) and back
                    (VideoTypeEnum::Image(v), VideoChangesReplaceWith::Text) => {
                        VideoTypeEnum::Text(crate::content::text::Text::new(
                            crate::content::text::TextType::Static(
                                v.path().to_string_lossy().to_string(),
                            ),
                            self.generic_content_data.clone(),
                        ))
                    }
                    (VideoTypeEnum::Raw(v), VideoChangesReplaceWith::Text) => {
                        VideoTypeEnum::Text(crate::content::text::Text::new(
                            crate::content::text::TextType::Static(
                                v.get_dir().to_string_lossy().to_string(),
                            ),
                            self.generic_content_data.clone(),
                        ))
                    }
                    (VideoTypeEnum::Ffmpeg(v), VideoChangesReplaceWith::Text) => {
                        VideoTypeEnum::Text(crate::content::text::Text::new(
                            crate::content::text::TextType::Static(
                                v.path().to_string_lossy().to_string(),
                            ),
                            self.generic_content_data.clone(),
                        ))
                    }
                    (VideoTypeEnum::Text(t), into) => {
                        let text = match t.text() {
                            crate::content::text::TextType::Static(t) => t.clone(),
                            crate::content::text::TextType::Program(p) => {
                                p.path.to_string_lossy().to_string()
                            }
                        };
                        match into {
                            VideoChangesReplaceWith::List => VideoTypeEnum::List(vec![]),
                            VideoChangesReplaceWith::AspectRatio => VideoTypeEnum::AspectRatio(
                                Box::new(Video::new_full(VideoType::new(
                                    VideoTypeEnum::List(vec![]),
                                    self.generic_content_data.clone(),
                                ))),
                                CurveData::Constant(1.0).into(),
                                CurveData::Constant(1.0).into(),
                            ),
                            VideoChangesReplaceWith::WithEffect => VideoTypeEnum::WithEffect(
                                Box::new(Video::new_full(VideoType::new(
                                    VideoTypeEnum::List(vec![]),
                                    self.generic_content_data.clone(),
                                ))),
                                crate::effect::Effect::new_from_enum(
                                    crate::effect::effects::EffectsEnum::Nothing(
                                        crate::effect::effects::Nothing::new(),
                                    ),
                                ),
                            ),
                            VideoChangesReplaceWith::Text => unreachable!(),
                            VideoChangesReplaceWith::Image => {
                                VideoTypeEnum::Image(crate::content::image::Image::new(
                                    text.into(),
                                    self.generic_content_data.reset(),
                                ))
                            }
                            VideoChangesReplaceWith::Raw => unreachable!(),
                            VideoChangesReplaceWith::Ffmpeg => {
                                VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(
                                    text.into(),
                                    self.generic_content_data.reset(),
                                ))
                            }
                        }
                    }
                    // don't use any information of the old one
                    (_, VideoChangesReplaceWith::List) => VideoTypeEnum::List(vec![]),
                    (_, VideoChangesReplaceWith::AspectRatio) => VideoTypeEnum::AspectRatio(
                        Box::new(Video::new_full(VideoType::new(
                            VideoTypeEnum::List(vec![]),
                            self.generic_content_data.clone(),
                        ))),
                        CurveData::Constant(1.0).into(),
                        CurveData::Constant(1.0).into(),
                    ),
                    (_, VideoChangesReplaceWith::WithEffect) => VideoTypeEnum::WithEffect(
                        Box::new(Video::new_full(VideoType::new(
                            VideoTypeEnum::List(vec![]),
                            self.generic_content_data.clone(),
                        ))),
                        crate::effect::Effect::new_from_enum(
                            crate::effect::effects::EffectsEnum::Nothing(
                                crate::effect::effects::Nothing::new(),
                            ),
                        ),
                    ),
                    (_, VideoChangesReplaceWith::Text) => {
                        VideoTypeEnum::Text(crate::content::text::Text::new(
                            crate::content::text::TextType::Static("[some text]".to_string()),
                            self.generic_content_data.clone(),
                        ))
                    }
                    (_, VideoChangesReplaceWith::Image) => {
                        VideoTypeEnum::Image(crate::content::image::Image::new(
                            "[image]".into(),
                            self.generic_content_data.reset(),
                        ))
                    }
                    (_, VideoChangesReplaceWith::Ffmpeg) => {
                        VideoTypeEnum::Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid::new(
                            "[video]".into(),
                            self.generic_content_data.reset(),
                        ))
                    }
                };
                self.video.vt = new;
                out = true;
            }
        }
        out && !err
    }

    fn generic_content_data(&mut self) -> &mut crate::content::content::GenericContentData {
        &mut self.generic_content_data
    }
}
impl Video {
    pub fn new_full(video: VideoType) -> Self {
        Self {
            set_pos: Pos {
                align: PosAlign::Center,
                x: CurveData::Constant(0.5).into(),
                y: CurveData::Constant(0.5).into(),
                w: CurveData::Constant(1.0).into(),
                h: CurveData::Constant(1.0).into(),
            },
            set_start_frame: 0.0,
            set_length: 1.0,
            compositing: None,
            generic_content_data: video.generic_content_data.reset(),
            video,
            as_content_changes: VideoChanges::default(),
        }
    }
    pub fn new_full_size(start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: Pos {
                align: PosAlign::Center,
                x: CurveData::Constant(0.5).into(),
                y: CurveData::Constant(0.5).into(),
                w: CurveData::Constant(1.0).into(),
                h: CurveData::Constant(1.0).into(),
            },
            set_start_frame: start_frame,
            set_length: length,
            compositing: None,
            generic_content_data: video.generic_content_data.clone(),
            video,
            as_content_changes: VideoChanges::default(),
        }
    }
    pub fn new(pos: Pos<Curve, Curve>, start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: pos,
            set_start_frame: start_frame,
            set_length: length,
            compositing: None,
            generic_content_data: video.generic_content_data.clone(),
            video,
            as_content_changes: VideoChanges::default(),
        }
    }

    pub fn prep_draw(
        &self,
        outer_progress: f64,
        parent_data: Option<&PrepDrawData>,
    ) -> Option<PrepDrawData> {
        // handle outer_progress
        if outer_progress < self.set_start_frame {
            return None;
        };
        let frames_since_start = outer_progress - self.set_start_frame;
        if frames_since_start >= self.set_length {
            return None;
        };
        if self.set_length <= 0.0 {
            return None;
        }
        let progress = frames_since_start / self.set_length;
        //
        let position = self.set_pos.convert(&|c| c.get_value(progress));
        let pos_px = if let Some(pd) = parent_data {
            Some(pd.pos_px)
        } else {
            None
        };
        let mut pdd = PrepDrawData {
            pos_px_from_canvas: pos_px.is_none(),
            pos_px: match pos_px {
                None => (0.0, 0.0, 0.0, 0.0),
                Some(v) => v,
            },
            position,
            progress,
            compositing: if let Some(v) = &self.compositing {
                v.clone()
            } else {
                if let Some(pd) = parent_data {
                    pd.compositing.clone()
                } else {
                    CompositingMethod::Ignore
                }
            },
            _private: (),
        };
        if pos_px.is_some() {
            pdd.calc_pos_px();
        }
        Some(pdd)
    }
}
pub struct PrepDrawData {
    pub progress: f64,
    pub position: Pos<f64, f64>,
    pub pos_px: (f64, f64, f64, f64),
    pub pos_px_from_canvas: bool,
    pub compositing: CompositingMethod,
    /// This prevents construction of this struct
    _private: (),
}
impl PrepDrawData {
    /// if pos_px is set to the parent's values, this function changes it to the correct ones (based on position)
    pub fn calc_pos_px(&mut self) {
        let tl = self.position.top_left_xy();
        self.pos_px = (
            self.pos_px.0 + self.pos_px.2 * tl.0,
            self.pos_px.1 + self.pos_px.3 * tl.1,
            self.pos_px.2 * self.position.w,
            self.pos_px.3 * self.position.h,
        )
    }
}

impl Video {
    /// This may only be called after prep_draw (which is why it consumes PrepDrawData).
    /// Between prep_draw and draw, effects can make some changes to the PrepDrawData.
    pub fn draw(
        &mut self,
        img: &mut DynamicImage,
        mut prep_data: PrepDrawData,
        render_settings: &mut VideoRenderSettings,
    ) {
        if prep_data.pos_px_from_canvas {
            prep_data.pos_px = (0.0, 0.0, img.width() as _, img.height() as _);
            prep_data.calc_pos_px();
        }
        self.draw2(img, prep_data, render_settings);
    }

    fn draw2(
        &mut self,
        image: &mut DynamicImage,
        prep_data: PrepDrawData,
        render_settings: &mut VideoRenderSettings,
    ) {
        // Rendering
        self.video.draw_on(image, prep_data, render_settings);
    }
}

pub trait Drawable {
    /// Make sure to respect the compositing method and position in prep_data!
    fn draw_on(
        &mut self,
        image: &mut DynamicImage,
        prep_data: PrepDrawData,
        render_settings: &mut VideoRenderSettings,
    );
}

#[derive(Clone)]
pub enum CompositingMethod {
    Ignore,
    /// Writes the pixels directly to the output buffer. Any color data that was previously there will be destroyed.
    Opaque,
    /// Like Opaque, but writes the alpha channel to the output buffer.
    Direct,
    /// Based on the alpha value of each pixel, merges what was there before with what is there now.
    TransparencySupport,
    Manual(crate::external_program::ExternalProgram),
}

// #[derive(Clone)]
// pub enum TransparencyAdjustments<T> {
//     None,
//     Force(T),
//     Factor(T),
//     ForceOpaqueIfNotTransparent,
// }
// impl<T> TransparencyAdjustments<T> {
//     pub fn convert<F, R>(self, f: &F) -> TransparencyAdjustments<R> where F: Fn(T) -> R {
//         match self {
//             Self::None => TransparencyAdjustments::None,
//             Self::Force(t) => TransparencyAdjustments::Force(f(t)),
//             Self::Factor(t) => TransparencyAdjustments::Factor(f(t)),
//             Self::ForceOpaqueIfNotTransparent => TransparencyAdjustments::ForceOpaqueIfNotTransparent,
//         }
//     }
// }

impl Video {
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
}
impl VideoType {
    pub fn new(vt: VideoTypeEnum, generic_content_data: GenericContentData) -> Self {
        Self {
            vt,
            changes: None,
            generic_content_data,
        }
    }
}
impl Drawable for VideoType {
    fn draw_on(
        &mut self,
        image: &mut DynamicImage,
        prep_data: PrepDrawData,
        render_settings: &mut VideoRenderSettings,
    ) {
        self.vt.draw_on(image, prep_data, render_settings);
    }
}
pub enum VideoTypeEnum {
    List(Vec<Video>),
    AspectRatio(Box<Video>, Curve, Curve),
    WithEffect(Box<Video>, crate::effect::Effect),
    Text(crate::content::text::Text),
    Image(crate::content::image::Image),
    Raw(crate::content::input_video::InputVideo),
    Ffmpeg(crate::content::ffmpeg_vid::FfmpegVid),
}

impl Drawable for VideoTypeEnum {
    fn draw_on(
        &mut self,
        image: &mut DynamicImage,
        prep_data: PrepDrawData,
        render_settings: &mut VideoRenderSettings,
    ) {
        match self {
            Self::Raw(raw_img) => {
                //println!("Drawing RAW");
                let img = raw_img.get_frame_fast(
                    prep_data.progress,
                    render_settings.max_distance_when_retrieving_closest_frame,
                );
                if let Some(img) = img {
                    img.draw(image, &prep_data, render_settings.image_scaling_filter_type);
                };
            }

            Self::Ffmpeg(vid) => {
                vid.load_img_force_factor(prep_data.progress);
                vid.draw(image, &prep_data, render_settings.image_scaling_filter_type)
            }

            Self::Image(img) => {
                img.draw(image, &prep_data, render_settings.image_scaling_filter_type)
            }

            Self::Text(txt) => txt.draw(
                image,
                &prep_data,
                prep_data.position.align.get_anchor(0.0, 0.5, 1.0),
            ),

            Self::WithEffect(vid, effect) => {
                effect.process_image(prep_data.progress, vid, image, render_settings, &prep_data);
            }

            Self::List(others) => match prep_data.compositing {
                CompositingMethod::Ignore => (),
                _ => {
                    for other in others {
                        if let Some(prep_draw) =
                            other.prep_draw(prep_data.progress, Some(&prep_data))
                        {
                            other.draw(image, prep_draw, render_settings);
                        }
                    }
                }
            },

            Self::AspectRatio(vid, width, height) => {
                if let Some(mut prep_draw) = vid.prep_draw(prep_data.progress, Some(&prep_data)) {
                    let my_aspect_ratio = render_settings.this_frame.out_vid_aspect_ratio
                        * (prep_draw.pos_px.2)
                        / (prep_draw.pos_px.3);
                    // println!("My AR: {}", my_aspect_ratio);
                    let (width, height) = (
                        width.get_value(prep_data.progress),
                        height.get_value(prep_data.progress),
                    );
                    if height != 0.0 {
                        let desired_aspect_ratio = width / height;
                        let (anchor_x, anchor_y) =
                            prep_draw.position.align.get_anchor(0.0, 0.5, 1.0);
                        if my_aspect_ratio > desired_aspect_ratio {
                            // too wide
                            let w = desired_aspect_ratio / my_aspect_ratio; // < 1
                            let x = (1.0 - w) * anchor_x;
                            prep_draw.position.w *= w;
                            prep_draw.position.x = x + prep_draw.position.x * w;
                        } else if my_aspect_ratio < desired_aspect_ratio {
                            // too high
                            let h = my_aspect_ratio / desired_aspect_ratio; // < 1
                            let y = (1.0 - h) * anchor_y;
                            prep_draw.position.h *= h;
                            prep_draw.position.y = y + prep_draw.position.y * h;
                        }
                    }
                    vid.draw(image, prep_draw, render_settings);
                };
            }
        }
    }
}

/// Draws the image onto the other one, following the compositing method and position provided by prep_draw. ASSUMES THE IMAGE HAS THE CORRECT SIZE!
pub fn composite_images(image: &mut DynamicImage, img: &DynamicImage, prep_draw: &PrepDrawData) {
    let pos = (
        prep_draw.pos_px.0.ceil() as i32,
        prep_draw.pos_px.1.ceil() as i32,
        prep_draw.pos_px.2 as u32,
        prep_draw.pos_px.3 as u32,
    );
    match &prep_draw.compositing {
        CompositingMethod::Ignore => (),
        CompositingMethod::Opaque => {
            for pixel in img.to_rgba8().enumerate_pixels() {
                let (x, y, pixel) = (pixel.0, pixel.1, pixel.2);
                let x = pos.0 + x as i32;
                let y = pos.1 + y as i32;
                if !x.is_negative()
                    && !y.is_negative()
                    && (x as u32) < image.width()
                    && (y as u32) < image.height()
                {
                    image.put_pixel(
                        x as _,
                        y as _,
                        image::Rgba {
                            0: [pixel.0[0], pixel.0[1], pixel.0[2], 255],
                        },
                    );
                }
            }
        }
        CompositingMethod::Direct => {
            for pixel in img.to_rgba8().enumerate_pixels() {
                let (x, y, pixel) = (pixel.0, pixel.1, pixel.2);
                let x = pos.0 + x as i32;
                let y = pos.1 + y as i32;
                if !x.is_negative()
                    && !y.is_negative()
                    && (x as u32) < image.width()
                    && (y as u32) < image.height()
                {
                    image.put_pixel(x as _, y as _, pixel.clone());
                }
            }
        }
        CompositingMethod::TransparencySupport => {
            for pixel in img.to_rgba8().enumerate_pixels() {
                let (x, y, pixel) = (pixel.0, pixel.1, pixel.2);
                let x = pos.0 + x as i32;
                let y = pos.1 + y as i32;
                if !x.is_negative()
                    && !y.is_negative()
                    && (x as u32) < image.width()
                    && (y as u32) < image.height()
                {
                    let mut px = image.get_pixel(x as _, y as _).clone();
                    composite_pixels_transparency_support(&mut px.0, &pixel.0);
                    image.put_pixel(x as _, y as _, px);
                }
            }
        }
        CompositingMethod::Manual(_) => todo!("Manual compositing not yet supported for images."),
    }
}

/// Draws onto old. This is to be used for CompositingMethod::TransparencySupport.
pub fn composite_pixels_transparency_support(old: &mut [u8; 4], new: &[u8; 4]) {
    let factor_new = new[3] as u16;
    let factor_old = 255 - factor_new;
    for i in 0..3 {
        old[i] = ((old[i] as u16 * factor_old + new[i] as u16 * factor_new) / 255) as u8;
    }
    old[3] = 255;
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
        VideoType::new(
            match &self.vt {
                VideoTypeEnum::List(vec) => VideoTypeEnum::List({
                    let mut nvec = Vec::with_capacity(vec.len());
                    for v in vec {
                        nvec.push(v.clone_no_caching());
                    }
                    nvec
                }),
                VideoTypeEnum::AspectRatio(v, width, height) => VideoTypeEnum::AspectRatio(
                    Box::new(v.clone_no_caching()),
                    width.clone(),
                    height.clone(),
                ),
                VideoTypeEnum::WithEffect(v, e) => {
                    VideoTypeEnum::WithEffect(Box::new(v.clone_no_caching()), e.clone_no_caching())
                }
                VideoTypeEnum::Text(t) => VideoTypeEnum::Text(t.clone_no_caching()),
                VideoTypeEnum::Image(img) => VideoTypeEnum::Image(img.clone_no_caching()),
                VideoTypeEnum::Raw(v) => VideoTypeEnum::Raw(v.clone_no_caching()),
                VideoTypeEnum::Ffmpeg(v) => VideoTypeEnum::Ffmpeg(v.clone_no_caching()),
            },
            self.generic_content_data.reset(),
        )
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
                            VideoTypeChanges_List::Swap(a, b) => {
                                vt.swap(a, b);
                            }
                            VideoTypeChanges_List::Move(a, b) => {
                                let v = vt.remove(b);
                                vt.insert(a, v);
                            }
                            VideoTypeChanges_List::Insert(index, new_val) => {
                                vt.insert(index, new_val);
                            }
                            VideoTypeChanges_List::Push(new_val) => {
                                vt.push(new_val);
                            }
                            VideoTypeChanges_List::Change(index, changes) => {
                                let vid = vt.get_mut(index).unwrap();
                                vid.apply_changes();
                                vid.as_content_changes = changes;
                                vid.apply_changes();
                            }
                            VideoTypeChanges_List::Replace(index, new_val) => {
                                *vt.get_mut(index).unwrap() = new_val
                            }
                            VideoTypeChanges_List::Remove(index) => {
                                vt.remove(index);
                            }
                        };
                    }
                    true
                }
                (
                    VideoTypeChanges::AspectRatio(vid_changes, width, height),
                    VideoTypeEnum::AspectRatio(v, w, h),
                ) => {
                    let mut out = true;
                    if let Some(changes) = vid_changes {
                        v.as_content_changes = *changes;
                        if !v.apply_changes() {
                            out = false;
                        };
                    }
                    if let Some(width) = width {
                        *w = width;
                    }
                    if let Some(height) = height {
                        *h = height;
                    }
                    out
                }
                (
                    VideoTypeChanges::WithEffect(vid_changes, eff_new),
                    VideoTypeEnum::WithEffect(vid, eff),
                ) => {
                    let mut out = true;
                    if let Some(changes) = vid_changes {
                        vid.as_content_changes = *changes;
                        if !vid.apply_changes() {
                            out = false;
                        };
                    };
                    if let Some(eff_new) = eff_new {
                        *eff = eff_new;
                    };
                    out
                }
                (VideoTypeChanges::Image(img_changes), VideoTypeEnum::Image(img)) => {
                    img.as_content_changes = img_changes;
                    img.apply_changes()
                }
                (VideoTypeChanges::Raw(changes), VideoTypeEnum::Raw(vid)) => {
                    vid.as_content_changes = changes;
                    vid.apply_changes()
                }
                (VideoTypeChanges::ChangeType(new), _) => {
                    self.vt = new;
                    true
                }
                (changes, data) => panic!(
                    "\n{}\n    {}{}{}{}{}\n",
                    Clz::error_info(
                        "Attempted to apply VideoTypeChanges, but found different types:"
                    ),
                    Clz::error_details("Tried to apply changes of type "),
                    Clz::error_cause(match changes {
                        VideoTypeChanges::ChangeType(_) => "[?]",
                        VideoTypeChanges::List(_) => "List",
                        VideoTypeChanges::AspectRatio(_, _, _) => "AspectRatio",
                        VideoTypeChanges::WithEffect(_, _) => "WithEffect",
                        VideoTypeChanges::Text(_) => "Text",
                        VideoTypeChanges::Image(_) => "Image",
                        VideoTypeChanges::Raw(_) => "Video",
                        VideoTypeChanges::Ffmpeg(_) => "ffmpeg",
                    }),
                    Clz::error_details(" to data of type "),
                    Clz::error_cause(match data {
                        VideoTypeEnum::List(_) => "List",
                        VideoTypeEnum::AspectRatio(_, _, _) => "AspectRatio",
                        VideoTypeEnum::WithEffect(_, _) => "WithEffect",
                        VideoTypeEnum::Text(_) => "Text",
                        VideoTypeEnum::Image(_) => "Image",
                        VideoTypeEnum::Raw(_) => "Video",
                        VideoTypeEnum::Ffmpeg(_) => "ffmpeg",
                    }),
                    Clz::error_details(".")
                ),
            }
        } else {
            true
        }
    }

    fn generic_content_data(&mut self) -> &mut crate::content::content::GenericContentData {
        &mut self.generic_content_data
    }
}

#[derive(Clone)]
pub struct Pos<T, U>
where
    T: Sized + Clone,
    U: Sized + Clone,
{
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
    fn convert<U, F>(&self, converter: F) -> PosAlign<U>
    where
        F: Fn(&T) -> U,
    {
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
    fn get_anchor(&self, a: T, b: T, c: T) -> (T, T)
    where
        T: Clone,
    {
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

impl<T, U> Pos<T, U>
where
    T: Clone,
    U: Clone,
{
    /// Converts all values from T or U to A, including the ones in self.align
    pub fn convert<A, F>(&self, converter: &F) -> Pos<A, A>
    where
        F: Fn(&T) -> A + Fn(&U) -> A,
        A: Clone,
    {
        self.convert_sep(converter, converter, |c| c.convert(converter))
    }

    pub fn convert_sep<A, B, F, G, H>(
        &self,
        converter_pos: F,
        converter_dimen: G,
        converter_align: H,
    ) -> Pos<A, B>
    where
        F: Fn(&T) -> A,
        G: Fn(&U) -> B,
        H: Fn(&PosAlign<T>) -> PosAlign<A>,
        A: Clone,
        B: Clone,
    {
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
        (
            match self.align {
                PosAlign::TopLeft | PosAlign::Left | PosAlign::BottomLeft => self.x, // left
                PosAlign::Top | PosAlign::Center | PosAlign::Bottom => self.x - self.w / 2.0, // mid
                PosAlign::TopRight | PosAlign::Right | PosAlign::BottomRight => self.x - self.w, // right
                PosAlign::Custom(v, _) => self.x - v * self.w,
            },
            match self.align {
                PosAlign::TopLeft | PosAlign::Top | PosAlign::TopRight => self.y, // top
                PosAlign::Left | PosAlign::Center | PosAlign::Right => self.y - self.h / 2.0, // mid
                PosAlign::BottomLeft | PosAlign::Bottom | PosAlign::BottomRight => self.y - self.h, // bottom
                PosAlign::Custom(_, v) => self.y - v * self.h,
            },
        )
    }
}
