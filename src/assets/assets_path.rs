use std::path::PathBuf;

use super::AssetsManager;

impl AssetsManager {
    pub fn get_assets_path(&mut self) -> PathBuf {
        if let None = self.assets_path {
            panic!("Could not open assets directory - please specify it using --assets-dir!");
        };
        self.assets_path.clone().unwrap()
    }
}
