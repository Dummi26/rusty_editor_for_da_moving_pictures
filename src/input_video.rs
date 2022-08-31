use std::{fs::{self, File}, io::{self, Read, Cursor}, path::PathBuf};

use image::{io::Reader as ImageReader, DynamicImage};

pub struct InputVideo {
    images_directory: PathBuf,
    /// A vec containing the image data. The length of the vec is the amount of frames available.
    /// FrameData: raw image data for all given frames.
    /// Option<i8>: For every frame, if this is 0, the frame is available and ready to be used. If it is positive, the nearest available frame is n entries further down the vec. If it is negative, the same thing applies, but in the opposite direction. If it is none, there is no value close enough.
    frames_image_data: Vec<(Option<i8>, crate::content::image::Image)>,
}
impl InputVideo {
    /// You can use "ffmpeg -i vids/video.mp4 path/%09d.png" or something similar to generate such a directory. Make sure the path ends in the path separator (likely \ on windows and / on unix)!
    pub fn new_from_directory_full_of_frames(images_directory: PathBuf) -> Result<Self, io::Error> {
        let dir_files_iter = fs::read_dir(&images_directory)?;
        let mut frames_image_data = Vec::new();
        for pot_file in dir_files_iter {
            let path = pot_file?.path();
            if path.is_file() {
                frames_image_data.push(path.file_name().unwrap().to_string_lossy().to_string());
            };
        };
        frames_image_data.sort_unstable();
        let frames_image_data = Vec::from_iter(frames_image_data.into_iter()
            .map(|file_name| (None, crate::content::image::Image::new({ let mut p = images_directory.clone(); p.push(file_name); p })))
        );
        Ok(Self {
            images_directory: images_directory,
            frames_image_data,
        })
    }

    pub fn get_length(&self) -> usize {
        self.frames_image_data.len()
    }
    /// Equivalent to get_frame_fast with max_frames_distance = 0.
    pub fn get_frame<'a>(&'a mut self, progress: f64) -> &'a mut crate::content::image::Image {
        self.get_frame_fast(progress, 0)
    }
    /// If there is a frame that has already been loaded near the current frame, use that frame instead. DO NOT USE THIS FOR RENDERING THE FINAL VIDEO - IT WILL SKIP FRAMES WHENEVER IT POSSIBLY CAN!
    pub fn get_frame_fast<'a>(&'a mut self, progress: f64, max_frames_distance: i8) -> &'a mut crate::content::image::Image {
        let mut index = ((self.get_length()-1) as f64 * progress).round() as usize;
        let offset = self.frames_image_data[index].0;
        if if let Some(o) = offset {
            let o_abs = o.abs();
            if o_abs <= max_frames_distance {
                if o == 0 {
                } else if o > 0 {
                    index += o_abs as usize;
                } else if o < 0 {
                    index -= o_abs as usize;
                };
                true
            } else {
                false
            }
        } else {
            false
        } {
            // close enough, index was adjusted if necessary
            &mut self.frames_image_data[index].1
        } else {
            self.load_image_at_index(index)
        }
    }
    fn load_image_at_index<'a>(&'a mut self, index: usize) -> &'a mut crate::content::image::Image {
        /* adjust the distances */ {
            for dist in 0..127.min(index) as i8 {
                let nindex = index - dist as usize;
                if let Some(prev_dist) = self.frames_image_data[nindex].0 { if prev_dist.abs() <= dist { break; } };
                self.frames_image_data[nindex].0 = Some(dist);
            };
            for dist in 1..127.min(self.frames_image_data.len() - index) as i8 {
                let nindex = index + dist as usize;
                if let Some(prev_dist) = self.frames_image_data[nindex].0 { if prev_dist.abs() <= dist { break; } };
                self.frames_image_data[nindex].0 = Some(-dist);
            };
        };
        // return the image
        &mut self.frames_image_data[index].1
    }
}