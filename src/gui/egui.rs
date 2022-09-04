use std::path::PathBuf;

use crate::cli::Clz;
use crate::video::VideoTypeEnum;
use eframe::egui;

use crate::project::Project;
use crate::video_cached_frames::{VideoCachedFramesOfCertainResolution};
use crate::multithreading::automatically_cache_frames::VideoWithAutoCache;

pub fn main(args: crate::cli::CustomArgs) -> ! {
    let path = match args.project_path {
        Some(v) => v,
        None => panic!("\n{}\n",
            Clz::error_info("Could not launch gui because no project was specified. Please use --proj-path to point to a project file you would like to open."),
        ),
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty editor",
        options,
        Box::new(|_cc| Box::new(MainWindow::new(path.into()))),
    );
}

struct MainWindow {
    project: Project,
    video_cache: VideoCachedFramesOfCertainResolution,
    /// The video progress chosen by the user.
    video_progress_fast: f64,
    /// The video progress from where the video is actually loaded. This might not match video_progress_fast for performance and caching reasons.
    video_progress_exact: f64,
    /// The video progress when the video was previously updated. If video_progress != Some(this), the video will be updated. If this is none, the video will also be updated. After the video is updated, this will be set to Some(video_progress).
    video_progress_fast_prev: Option<f64>,
    video_progress_exact_prev: Option<f64>,
    video_preview: VideoPreview,
}
impl MainWindow {
    fn new(project_file: PathBuf) -> Self {
        let video = VideoWithAutoCache::new(
            crate::video::Video::new_full(crate::video::VideoType::new(VideoTypeEnum::List(Vec::new()))));
        Self {
            video_cache: Self::get_video_cache(&video),
            project: {
                let mut proj = match crate::files::file_handler::read_from_file(&project_file) {
                    Ok(Ok(v)) => v,
                    Ok(Err(err)) => panic!("[ParseFile @ egui::MainWindow::new(..)] Error parsing file: {}", err.to_string()),
                    Err(err) => panic!("[ParseFile @ egui::MainWindow::new(..)] IO Error: {}", err),
                };
                println!("Loaded!");
                proj.vid.cache();
                proj
            },
            video_progress_fast: 0.0,
            video_progress_exact: 0.0,
            video_progress_fast_prev: None,
            video_progress_exact_prev: None,
            video_preview: Default::default(),
        }
    }
}
impl MainWindow {
    fn get_video_cache(video: &VideoWithAutoCache) -> VideoCachedFramesOfCertainResolution {
        let (width, height) = video.get_width_and_height_mutex();
        video.get_vid_mutex_arc().lock().unwrap().last_draw.with_resolution_or_create(width, height)
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.video_progress_fast, 0.0..=1.0));
            //ui.add(egui::Slider::new(&mut self.video_progress_exact, 0.0..=1.0));
            /**/ {
                let (w, h) = (ui.available_width().floor() as u32, ui.available_height().floor() as u32);
                if w != self.video_cache.width() || h != self.video_cache.height() {
                    self.project.vid.set_width_and_height(w, h);
                    self.video_cache = Self::get_video_cache(&self.project.vid);
                };
                if self.video_progress_fast_prev != Some(self.video_progress_fast) { // user changed 'fast' slider
                    self.video_progress_fast_prev = Some(self.video_progress_fast);
                    let frame = {
                        let vc = self.video_cache.cache().lock().unwrap();
                        let frame = vc.get_frame(self.video_progress_fast);
                        if let Some((dist, frame)) = frame {
                            Some((dist, frame.progress, frame.frame.to_rgba8()))
                        } else {
                            None
                        }
                    };
                    if let Some((_distance, progress, img)) = frame {
                        self.video_preview.image = Some(img);
                        self.video_progress_exact = progress;
                        self.video_progress_exact_prev = Some(progress);
                    };
                };
            };
            self.video_progress_exact_prev = Some(self.video_progress_exact);
            self.video_preview.ui(ui);
        });
    }
}

#[derive(Default)]
struct VideoPreview {
    image: Option<image::RgbaImage>,
    texture: Option<egui::TextureHandle>,
}
impl VideoPreview {
    fn ui(&mut self, ui: &mut egui::Ui) {
        match &self.image.take() {
            Some(img) => {
                self.texture = Some(ui.ctx().load_texture("video-preview-image",
                    egui::ColorImage::from_rgba_unmultiplied([img.width() as _, img.height() as _], img.as_raw().as_slice())
                ));
            },
            None => {},
        };

        // Show the image:
        if let Some(texture) = &self.texture {
            ui.add(egui::Image::new(texture, texture.size_vec2()));
        };

        // Shorter version:
        // ui.image(texture, texture.size_vec2());
    }
}