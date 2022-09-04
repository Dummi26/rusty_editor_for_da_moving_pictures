use std::path::PathBuf;

use speedy2d::font::Font;

mod fonts;
mod assets_path;

#[derive(Default)]
pub struct AssetsManager {
    assets_path: Option<PathBuf>,
    default_font: Option<Font>,
}