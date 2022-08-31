use std::{thread, time::{Duration, Instant}};

use colored::Colorize;
use curve::Curve;
use image::GenericImageView;
use video::Video;

use crate::{input_video::InputVideo, video::Pos, video_render_settings::VideoRenderSettings};

mod useful;
mod video;
mod video_render_settings;
mod video_cached_frames;
mod input_video;
mod content;
mod effect;
mod curve;
mod project;
mod gui;
mod multithreading;
mod files;

// ffmpeg -i vids/video.mp4 path/%09d.png

fn main() {
    if gui::IS_ENABLED {
        gui::main();
    }
}

struct Location {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl Location {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h, }
    }
    pub fn pos(&self, x: u32, y: u32) -> (u32, u32) {
        (x + self.x, y + self.y)
    }
}