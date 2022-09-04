use std::path::PathBuf;

use crate::{multithreading::automatically_cache_frames::VideoWithAutoCache, video_render_settings::VideoRenderSettings};

pub struct Project {
    pub proj: ProjectData,
    pub vid: VideoWithAutoCache,
}
pub struct ProjectData {
    pub name: String,
    pub path: Option<PathBuf>,
    pub render_settings_export: Option<VideoRenderSettings>,
}