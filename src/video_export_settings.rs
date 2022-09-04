use std::path::PathBuf;

pub struct VideoExportSettings {
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub frames: u32,
}