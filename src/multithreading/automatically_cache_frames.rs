use std::{thread::{self, JoinHandle}, time::{Duration, Instant}};

use crate::video::Video;
use std::sync::{Arc, Mutex};


pub struct VideoWithAutoCache {
    thread: JoinHandle<()>,
    sender: std::sync::mpsc::Sender<BackgroundThreadCommand>,
    pub shared: std::sync::Arc<std::sync::Mutex<SharedData>>,
}

/// This is the information the background thread has access to.
/// None of its fields should be operated on for more than a few milliseconds. Anything that takes longer should be behind an Arc<Mutex<..>> so that the main thread, when accessing this struct,
/// does not have to wait.
pub struct VideoWithAutoCacheData {
    pub commands: std::sync::mpsc::Receiver<BackgroundThreadCommand>,
    width: u32,
    height: u32,
    shared: std::sync::Arc<std::sync::Mutex<SharedData>>,
}
pub enum BackgroundThreadCommand {
    Stop,
    Pause(bool),
    Resume,
    SetResulution(u32, u32),
    SetProgress(f64),
}
impl VideoWithAutoCache {
    pub fn start(vid: Arc<Mutex<Video>>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let shared = std::sync::Arc::new(std::sync::Mutex::new(SharedData::default()));
        let thread = {
            let shared = shared.clone();
            thread::spawn(move || {
                eprintln!("[bg.c] Starting thread...");
                let sleep_duration = Duration::from_millis(25);
                let sleep_duration_while_paused = Duration::from_secs(1);
                // number of rendered frames
                let mut frames_count = 0u128;
                // 'outer indicates that breaking from this loop will immedeately end the thread.
                let (mut width, mut height) = (0, 0);
                let mut paused = false;
                let mut progress = 0.0;
                let mut should_render = true;
                'outer: loop {
                    // let start_time = Instant::now();
                    while let Ok(command) = receiver.try_recv() {
                        match command {
                            BackgroundThreadCommand::Stop => break 'outer,
                            BackgroundThreadCommand::Pause(clear) => { paused = true; if clear { shared.lock().unwrap().frame = None; /* TODO: clear cache */ } },
                            BackgroundThreadCommand::Resume => { paused = false; should_render = true; },
                            BackgroundThreadCommand::SetResulution(w, h) => (width, height) = (w, h),
                            BackgroundThreadCommand::SetProgress(prog) => if progress != prog { progress = prog; should_render = true; },
                        }
                    }
                    if paused {
                        thread::sleep(sleep_duration_while_paused);
                        continue;
                    }

                    if width != 0 && height != 0 {
                        if should_render {
                            should_render = false;
                            let mut img = Box::new(image::DynamicImage::new_rgb8(width, height));
                            {
                                let mut vid = vid.lock().unwrap();
                                if let Some(prep_data) = vid.prep_draw(progress) {
                                    vid.draw(img.as_mut(), prep_data, &mut crate::video_render_settings::VideoRenderSettings::preview());
                                    frames_count += 1;
                                };
                            };
                            shared.lock().unwrap().frame = Some(img);
                        }
                    };
                    thread::sleep(sleep_duration);
                };
            })
        };
        Self {
            thread,
            sender,
            shared,
        }
    }

    pub fn set_width_and_height(&mut self, width: u32, height: u32) -> bool {
        self.sender.send(BackgroundThreadCommand::SetResulution(width, height)).is_ok()
    }
    pub fn pause(&self, clear: bool) -> bool {
        self.sender.send(BackgroundThreadCommand::Pause(clear)).is_ok()
    }
    pub fn resume(&self) -> bool {
        self.sender.send(BackgroundThreadCommand::Resume).is_ok()
    }
    pub fn stop(&self) -> bool {
        self.sender.send(BackgroundThreadCommand::Stop).is_ok()
    }
    pub fn set_desired_progress(&self, prog: f64) -> bool {
        self.sender.send(BackgroundThreadCommand::SetProgress(prog)).is_ok()
    }

    pub fn is_still_alive(&self) -> bool {
        !self.thread.is_finished()
    }
    pub fn wait_for_thread_to_stop(self) -> bool {
        self.thread.join().is_ok()
    }
}

#[derive(Default)]
pub struct SharedData {
    pub frame: Option<Box<image::DynamicImage>>,
}