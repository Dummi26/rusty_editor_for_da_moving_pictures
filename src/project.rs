use std::path::PathBuf;

use crate::{video::Video, multithreading::automatically_cache_frames::VideoWithAutoCache};

pub struct Project {
    pub proj: ProjectData,
    pub vid: VideoWithAutoCache,
}
pub struct ProjectData {
    pub name: String,
    pub path: PathBuf,
}