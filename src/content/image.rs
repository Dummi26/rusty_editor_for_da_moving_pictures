use std::{path::PathBuf, io::{self, Read}};

use image::{DynamicImage, imageops::FilterType, GenericImageView, GenericImage};

pub struct Image {
    pub path: PathBuf,
    pub img_original: Option<DynamicImage>,
    pub img_scaled: Option<(FilterType, DynamicImage)>,
}
impl Image {
    pub fn new(path: PathBuf) -> Self {
        Self { path, img_original: None, img_scaled: None, }
    }
    pub fn load_img_force(&mut self) {
        self.img_original = match std::fs::File::open(&self.path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let io::Result::Err(err) = file.read_to_end(&mut buf) { panic!("While reading bytes from file, an error was encountered: {err}",); };
                match image::io::Reader::new(io::Cursor::new(buf)).with_guessed_format() {
                    Ok(img) => {
                        match img.decode() {
                            Ok(img) => Some(img),
                            Err(err) => panic!("Could not load image: {err}"),
                        }
                    },
                    Err(err) => panic!("Could not guess image format: {err}"),
                }
            },
            Err(err) => panic!("Could not open file at '{}': {}", &self.path.display(), err),
        }
    }
    pub fn load_img_if_necessary(&mut self) {
        if self.img_original.is_none() {
            self.load_img_force()
        }
    }
    pub fn get_img_original(&mut self) -> &DynamicImage {
        self.load_img_if_necessary();
        match &self.img_original { Some(v) => v, None => panic!("dadskjdklsanjkdsbads "), }
    }
    
    pub fn get_img_scaled(&mut self, width: u32, height: u32, scaling_filter: FilterType) -> &DynamicImage {
        if 
            if let Some((filter, img_scaled)) = &self.img_scaled {
                if *filter == scaling_filter && img_scaled.width() == width && img_scaled.height() == height {
                    false
                } else { true }
            } else { true }
        {
            self.img_scaled = Some((scaling_filter.clone(), self.get_img_original().resize_exact(width, height, scaling_filter)));
        };
        match &self.img_scaled { Some(v) => &v.1, None => panic!("djsakfhjdkfnfkl"), }
    }
    pub fn draw(&mut self, image: &mut DynamicImage, scaling_filter: FilterType) {
        let img = self.get_img_scaled(image.width(), image.height(), scaling_filter);
        //for pixel in img.pixels() { let (x, y, mut pixel) = (pixel.0, pixel.1, pixel.2); image.put_pixel(x, y, pixel); };
        *image = img.clone();
    }
}