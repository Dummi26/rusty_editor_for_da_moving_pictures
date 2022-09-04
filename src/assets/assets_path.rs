use std::path::PathBuf;

use super::AssetsManager;

impl AssetsManager {
    pub fn get_assets_path(&mut self) -> PathBuf {
        if let None = self.assets_path {
            self.assets_path = Some(find_path());
        };
        fn find_path() -> PathBuf {
            "/home/mark/Documents/code/rust/rusty_editor_for_da_moving_pictures/assets/".into()
        }
        self.assets_path.clone().unwrap()
    }
}