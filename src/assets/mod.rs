use std::path::PathBuf;

use speedy2d::font::Font;

mod fonts;
mod assets_path;

#[derive(Default)]
pub struct AssetsManager {
    pub assets_path: Option<PathBuf>,
    pub default_font: Option<Font>,
}
