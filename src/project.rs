use std::path::PathBuf;

use std::sync::{Arc, Mutex};
use crate::{video::Video, video_render_settings::VideoRenderSettings};

#[derive(Clone)]
pub struct Project {
    pub proj: Arc<Mutex<ProjectData>>,
    pub vid: Arc<Mutex<Video>>,
}
pub struct ProjectData {
    pub name: String,
    pub path: Option<PathBuf>,
    pub render_settings_export: Option<VideoRenderSettings>,
}