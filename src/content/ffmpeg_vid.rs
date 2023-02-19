use std::{
    io::{self, Read},
    path::PathBuf,
};

use image::{imageops::FilterType, DynamicImage};

use super::content::{Content, GenericContentData};

pub struct FfmpegVid {
    path: PathBuf,
    image: Option<DynamicImage>,
    generic_content_data: crate::content::content::GenericContentData,
    pub as_content_changes: FfmpegVidChanges,
}
#[derive(Default)]
pub struct FfmpegVidChanges {
    pub path: Option<PathBuf>,
}
impl Content for FfmpegVid {
    fn clone_no_caching(&self) -> Self {
        Self::new(self.path.clone(), self.generic_content_data.reset())
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
impl FfmpegVid {
    pub fn new(path: PathBuf, generic_content_data: GenericContentData) -> Self {
        Self {
            path,
            image: None,
            as_content_changes: FfmpegVidChanges::default(),
            generic_content_data,
        }
    }
}
impl FfmpegVid {
    pub fn set_path(&mut self, new: PathBuf) {
        self.path = new;
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_length_secs(&self) -> f64 {
        // ffprobe -v error -select_streams v:0 -show_entries stream=duration -of default=noprint_wrappers=1:nokey=1 ~/Videos/wat.mp4
        if let Ok(ffprobe_output) = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                self.path.to_string_lossy().as_ref(),
            ])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            if let Ok(s) = String::from_utf8(ffprobe_output.stdout) {
                if s.len() > 0 {
                    if let Ok(n) = s[0..s.len() - 1].parse() {
                        n
                    } else {
                        println!(
                            "ffprobe's output was not a valid float: '{}'",
                            &s[0..s.len() - 1]
                        );
                        0.0
                    }
                } else {
                    println!("ffprobe's output was an empty string.");
                    0.0
                }
            } else {
                println!("ffprobe's output was not utf-8.");
                0.0
            }
        } else {
            println!("Could not run ffprobe.");
            0.0
        }
    }
    pub fn load_img_force_factor(&mut self, factor: f64) {
        self.load_img_force_seconds(self.get_length_secs() * factor)
    }
    pub fn load_img_force_seconds(&mut self, secs: f64) {
        self.load_img_force(secs.to_string().as_str())
    }
    pub fn load_img_force(&mut self, time: &str) {
        // TODO: use different file names
        let image_file_path = "/tmp/dummi26/rusty_editor_for_da_moving_pictures/FfmpegVid_n.bmp";
        // ffmpeg -ss 00:01:00 -i ~/Videos/wat.mp4 -frames:v 1 /tmp/video/frame.png
        let ffmpeg_output = std::process::Command::new("ffmpeg")
            .args([
                "-ss",
                time.into(),
                "-i".into(),
                self.path.to_str().unwrap(),
                "-frames:v",
                "1",
                image_file_path,
            ])
            .stdin(std::process::Stdio::null())
            .output();
        match ffmpeg_output {
            Ok(s) => {
                if !s.status.success() {
                    println!(
                        "ffmpeg command failed:\nStdout:\n{}\nStderr:\n{}",
                        std::string::String::from_utf8_lossy(s.stdout.as_slice()),
                        std::string::String::from_utf8_lossy(s.stderr.as_slice())
                    );
                }
            }
            Err(e) => println!("ffmpeg command failed: {e}"),
        }
        let o = self.image = match std::fs::File::open(image_file_path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let io::Result::Err(err) = file.read_to_end(&mut buf) {
                    eprintln!("While reading bytes from file, an error was encountered: {err}",);
                    return;
                };
                match image::io::Reader::new(io::Cursor::new(buf)).with_guessed_format() {
                    Ok(img) => match img.decode() {
                        Ok(img) => Some(img.into_rgba8().into()),
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
                eprintln!("Could not open file at '{}': {}", image_file_path, err);
                None
            }
        };
        _ = std::fs::remove_file(image_file_path);
        o
    }
    pub fn get_img_scaled(
        &mut self,
        width: u32,
        height: u32,
        scaling_filter: FilterType,
    ) -> Option<DynamicImage> {
        match &self.image {
            Some(img) => Some(img.resize_exact(width, height, scaling_filter)),
            None => None,
        }
    }
    pub fn draw(
        &mut self,
        image: &mut DynamicImage,
        prep_draw: &crate::video::PrepDrawData,
        scaling_filter: FilterType,
    ) {
        let img = self.get_img_scaled(
            prep_draw.pos_px.2 as _,
            prep_draw.pos_px.3 as _,
            scaling_filter,
        );
        if let Some(img) = img {
            crate::video::composite_images(image, &img, prep_draw);
        };
    }
}
