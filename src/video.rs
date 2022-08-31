use std::sync::{Arc, Mutex};

use image::{DynamicImage, GenericImageView, GenericImage};

use crate::{curve::Curve, video_cached_frames::VideoCachedFrames, video_render_settings::VideoRenderSettings};

pub struct Video {
    // - - The video's data (what is to be saved to the project file) - -
    // set: Settings
    pub set_pos: Pos<Curve, Curve>,
    pub set_start_frame: f64,
    pub set_length: f64,
    /// The video
    pub video: VideoType,
    // - -     -     - -
    // done: The values that are set after drawing
    /// Due to caching, the rendered image might not be exactly the desired one. If this is the case, this value will differ from the progress used by draw() etc.
    pub done_actual_progress: f64,

    /// Caching for performance reasons (at the cost of memory usage)
    pub last_draw: VideoCachedFrames,
}
impl Video {
    pub fn new_full(video: VideoType) -> Self {
        Self {
            set_pos: Pos { x: Curve::Constant(0.0), y: Curve::Constant(0.0), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: 0.0,
            set_length: 1.0,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            video,
        }
    }
    pub fn new_full_size(start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: Pos { x: Curve::Constant(0.0), y: Curve::Constant(0.0), w: Curve::Constant(1.0), h: Curve::Constant(1.0) },
            set_start_frame: start_frame,
            set_length: length,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            video,
        }
    }
    pub fn new(pos: Pos<Curve, Curve>, start_frame: f64, length: f64, video: VideoType) -> Self {
        Self {
            set_pos: pos,
            set_start_frame: start_frame,
            set_length: length,
            done_actual_progress: 0.0,
            last_draw: VideoCachedFrames::new(),
            video,
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
            _private: (),
        })
    }
}
pub struct PrepDrawData {
    pub progress: f64,
    pub position: Pos<f64, f64>,
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
        self.draw2(img, prep_data.progress, pos, render_settings);
    }

    fn draw2(&mut self, image: &mut DynamicImage, progress: f64, pos: Pos<i32, u32>, render_settings: &VideoRenderSettings) {

        {
            let cached_frames_of_correct_resolution = if render_settings.allow_retrieval_of_cached_frames != None { self.last_draw.with_resolution(pos.w, pos.h) } else { None };
            match cached_frames_of_correct_resolution {
                Some(cache) => {
                    match cache.lock().unwrap().get_frame(progress) {
                        Some((dist, frame)) => {
                            if dist <= render_settings.allow_retrieval_of_cached_frames.expect("This always exists because for cfocr to be Some, arocf cannot be none.") {
                                self.done_actual_progress = frame.progress;
                                draw_to_canvas(image, &pos, &frame.frame);
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
            draw_to_canvas(image, &pos, &img);
            // Caching
            self.last_draw.add_frame(progress, pos.w, pos.h, img);
        };
        
        fn draw_to_canvas(image: &mut DynamicImage, pos: &Pos<i32, u32>, img: &DynamicImage) {

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
                image.put_pixel(x, y, pixel);
            };
        
        }
    }


    /// This function does not put the image into the cache automatically. To do that, use self.last_draw.add_frame(?, ?, ?, create_rendered_image(...))
    fn create_rendered_image(&mut self, pos: &Pos<i32, u32>, progress: f64, render_settings: &VideoRenderSettings) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(pos.w, pos.h);
        self.draw3(progress, render_settings, &mut img); // draw onto this image
        self.done_actual_progress = progress;
        img
    }


    /// crop: new min x and y (0 if not cropped), new right and bottom bounds (original width and height if not cropped or only cropped on left and/or top, otherwise width or height minus cropped pixels)
    fn draw3(&mut self, progress: f64, render_settings: &VideoRenderSettings, image: &mut DynamicImage) {

        match &mut self.video {



            VideoType::Raw(raw_img) => {
                //println!("Drawing RAW");
                let img = raw_img.get_frame_fast(progress, render_settings.max_distance_when_retrieving_closest_frame);
                img.draw(image, render_settings.image_scaling_filter_type);
            },
            
            
            
            VideoType::Image(img) => {
                img.draw(image, render_settings.image_scaling_filter_type);
            },



            VideoType::WithEffect(vid, effect) => {
                effect.process_image(progress, vid, image, render_settings);
            },



            VideoType::List(others) => {
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

pub enum VideoType {
    List(Vec<Video>),
    WithEffect(Box<Video>, crate::effect::Effect),
    Image(crate::content::image::Image),
    Raw(crate::input_video::InputVideo),
}

pub struct Pos<T, U> where T: Sized, U: Sized {
    pub x: T,
    pub y: T,
    pub w: U,
    pub h: U,
}
impl<T, U> Pos<T, U> {
    pub fn convert<A, F>(&self, converter: &F) -> Pos<A, A> where F: Fn(&T) -> A + Fn(&U) -> A {
        self.convert_sep(converter, converter)
    }
    pub fn convert_sep<A, B, F, G>(&self, converter1: F, converter2: G) -> Pos<A, B> where F: Fn(&T) -> A, G: Fn(&U) -> B {
        Pos {
            x: converter1(&self.x),
            y: converter1(&self.y),
            w: converter2(&self.w),
            h: converter2(&self.h),
        }
    }
}