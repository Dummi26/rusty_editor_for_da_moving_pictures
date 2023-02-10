use std::{
    io::{self, Read},
    path::PathBuf,
    process::Command,
};

use image::{imageops::FilterType, DynamicImage};

use super::content::Content;

pub struct Image {
    path: PathBuf,
    failed_to_load_image: bool,
    /// if set, every time an image should be displayed, this command is invoked. If a file is present at the given location, it will be (re)loaded as img_original.
    pub external_command: Option<(String, Vec<String>)>,
    pub external_command_replacements: Vec<(String, String)>,
    pub img_original: Option<DynamicImage>,
    pub img_scaled: Option<(FilterType, DynamicImage)>,
    generic_content_data: crate::content::content::GenericContentData,
    pub as_content_changes: ImageChanges,
}
#[derive(Default)]
pub struct ImageChanges {
    pub path: Option<PathBuf>,
}
impl Content for Image {
    fn clone_no_caching(&self) -> Self {
        Self::new(self.path.clone())
    }

    fn children(&self) -> Vec<&Self> {
        Vec::new()
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        Vec::new()
    }

    fn has_changes(&self) -> bool {
        self.as_content_changes.path.is_some()
    }
    fn apply_changes(&mut self) -> bool {
        if let Some(path) = self.as_content_changes.path.take() {
            self.set_path(path);
            true
        } else {
            false
        }
    }

    fn generic_content_data(&mut self) -> &mut super::content::GenericContentData {
        &mut self.generic_content_data
    }
}
impl Image {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            failed_to_load_image: false,
            external_command: None,
            external_command_replacements: Vec::new(),
            img_original: None,
            img_scaled: None,
            as_content_changes: ImageChanges::default(),
            generic_content_data: crate::content::content::GenericContentData::default(),
        }
    }
}
impl Image {
    pub fn set_path(&mut self, new: PathBuf) {
        self.img_original = None;
        self.img_scaled = None;
        self.path = new;
        self.failed_to_load_image = false;
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn load_img_force(&mut self) {
        self.img_original = match std::fs::File::open(&self.path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let io::Result::Err(err) = file.read_to_end(&mut buf) {
                    eprintln!("While reading bytes from file, an error was encountered: {err}",);
                    return;
                };
                match image::io::Reader::new(io::Cursor::new(buf)).with_guessed_format() {
                    Ok(img) => match img.decode() {
                        Ok(img) => Some(img),
                        Err(err) => {
                            eprintln!("Could not load image: {err}");
                            None
                        }
                    },
                    Err(err) => {
                        eprintln!("Could not guess image format: {err}");
                        None
                    }
                }
            }
            Err(err) => {
                eprintln!("Could not open file at '{}': {}", &self.path.display(), err);
                None
            }
        }
    }
    pub fn load_img_if_necessary(&mut self) {
        if let Some((command, args)) = &self.external_command {
            let mut command = Command::new(command);
            let args: Vec<_> = args
                .iter()
                .map(|arg| {
                    let mut arg = arg.to_string();
                    for (replace, with) in &self.external_command_replacements {
                        arg = arg.replace(replace, with);
                    }
                    arg
                })
                .collect();
            if !args.is_empty() {
                command.args(&args[..]);
            }
            match command.status() {
                Ok(s) if s.success() => (),
                s => eprintln!("Command status (image with custom command): {:?}", s),
            }
        }
        if self.external_command.is_some()
            || (self.img_original.is_none() && !self.failed_to_load_image)
        {
            self.load_img_force();
            if self.img_original.is_none() {
                self.failed_to_load_image = true;
            };
        };
    }
    pub fn get_img_original(&mut self) -> Option<&DynamicImage> {
        self.load_img_if_necessary();
        match &self.img_original {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn get_img_scaled(
        &mut self,
        width: u32,
        height: u32,
        scaling_filter: FilterType,
    ) -> Option<&DynamicImage> {
        if if let (false, Some((filter, img_scaled))) =
            (self.external_command.is_some(), &self.img_scaled)
        {
            if *filter == scaling_filter
                && img_scaled.width() == width
                && img_scaled.height() == height
            {
                false
            } else {
                true
            }
        } else {
            true
        } {
            if let Some(img_og) = self.get_img_original() {
                self.img_scaled = Some((
                    scaling_filter.clone(),
                    img_og.resize_exact(width, height, scaling_filter),
                ));
            };
        };
        match &self.img_scaled {
            Some(v) => Some(&v.1),
            None => None,
        }
    }
    pub fn draw(
        &mut self,
        image: &mut DynamicImage,
        prep_draw: &crate::video::PrepDrawData,
        scaling_filter: FilterType,
    ) {
        let (width, height): (u32, u32) = (prep_draw.pos_px.2 as _, prep_draw.pos_px.3 as _);
        if self.external_command.is_some() {
            self.external_command_replacements = vec![
                (
                    format!("%d26vid:progress%"),
                    format!("{}", prep_draw.progress),
                ),
                (format!("%d26vid:imgwidth%"), format!("{}", width)),
                (format!("%d26vid:imgheight%"), format!("{}", height)),
                (
                    format!("%d26vid:imgpath%"),
                    format!("{}", self.path().to_string_lossy().as_ref()),
                ),
            ];
        }
        let img = self.get_img_scaled(width, height, scaling_filter);
        if let Some(img) = img {
            crate::video::composite_images(image, img, prep_draw);
        };
    }
}
