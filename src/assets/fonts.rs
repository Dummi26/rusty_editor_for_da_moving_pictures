use std::{path::{Path, PathBuf}, fs, io};

use speedy2d::{font::Font, error::{BacktraceError, ErrorMessage}};

use crate::cli::Clz;

use super::AssetsManager;

impl AssetsManager {
    pub fn load_font<P>(&mut self, file: P) -> io::Result<Result<Font, BacktraceError<ErrorMessage>>> where P: AsRef<Path> {
        Ok(Font::new(fs::read(file)?.as_slice()))
    }
    pub fn get_default_font<'a>(&'a mut self) -> &'a mut Font {
        let mut io_err = None;
        let font_path =  self.get_default_font_path();
        match self.get_default_font_err() {
            Ok(Ok(font)) => return font,
            Err(err) => {
                println!("Io err!");
                io_err = Some(err);
            },
            Ok(Err(err)) => {
                panic!("\n{}{}\n",
                    Clz::error_info("Failed to load default font: "), Clz::error_details(err.to_string().as_str()),
                );
            },
        };
        if let Some(err) = io_err {
            let err = err.to_string();
            panic!("\n{}{}{}{}\n",
                Clz::error_info("Failed to load default font file '"), Clz::error_cause(font_path.to_string_lossy().as_ref()), Clz::error_info("': "), Clz::error_details(err.as_str()),
            );
        };
        panic!()
    }
    pub fn get_default_font_err<'a>(&'a mut self) -> io::Result<Result<&'a mut Font, BacktraceError<ErrorMessage>>> {
        if let None = self.default_font {
            let path = self.get_default_font_path();
            self.default_font = Some(match self.load_font(&path)? { Ok(v) => v, Err(err) => return Ok(Err(err)), });
        };
        Ok(Ok(match &mut self.default_font { Some(v) => v, None => panic!("Font did not exist, although it should have been loaded in this very function right here...?"), }))
    }
    pub fn get_default_font_path(&mut self) -> PathBuf {
        Self::get_default_font_path_from_assets_path(&self.get_assets_path())
    }
    pub fn get_default_font_path_from_assets_path(assets_path: &PathBuf) -> PathBuf {
        let mut p = assets_path.clone();
        p.push("fonts");
        p.push("FiraSans-Regular.ttf");
        p
    }
}