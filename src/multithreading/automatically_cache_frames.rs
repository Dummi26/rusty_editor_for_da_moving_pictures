use std::{thread::{self, JoinHandle}, time::Duration};

use crate::video::Video;
use std::sync::{Arc, Mutex};

pub struct VideoWithAutoCache {
    data: Arc<Mutex<VideoWithAutoCacheData>>,
    thread: Option<JoinHandle<()>>,
}

/// This is the information the background thread has access to.
/// None of its fields should be operated on for more than a few milliseconds. Anything that takes longer should be behind an Arc<Mutex<..>> so that the main thread, when accessing this struct,
/// does not have to wait.
struct VideoWithAutoCacheData {
    vid: Arc<Mutex<Video>>,
    commands: Vec<BackgroundThreadCommand>,
    width: u32,
    height: u32,
}
enum BackgroundThreadCommand {
    Stop,
}
impl VideoWithAutoCache {
    pub fn new(vid: Video, width: u32, height: u32) -> Self {
        Self { data: Arc::new(Mutex::new(VideoWithAutoCacheData { vid: Arc::new(Mutex::new(vid)), width, height, commands: Vec::new(), })), thread: None, }
    }
    
    pub fn set_width_and_height(&mut self, width: u32, height: u32) {
        let mut data = self.data.lock().unwrap();
        data.width = width;
        data.height = height;
    }

    pub fn get_width_and_height_mutex(&self) -> (u32, u32) { let data = self.data.lock().unwrap(); (data.width, data.height) }

    pub fn get_vid_mutex_arc(&self) -> Arc<Mutex<Video>> {
        self.data.lock().unwrap().vid.clone()
    }
    
    pub fn is_still_alive(&mut self) -> bool {
        if let Some(thread) = &self.thread {
            if thread.is_finished() {
                self.thread = None;
                false
            } else {
                true
            }
        } else {
            false
        }
    }
    pub fn signal_to_stop(&mut self) -> bool {
        if let Some(_) = &self.thread {
            self.data.lock().unwrap().commands.push(BackgroundThreadCommand::Stop);
            true
        } else {
            false
        }
    }
    pub fn wait_for_thread_to_stop(&mut self) -> bool {
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
    pub fn signal_to_and_wait_for_thread_to_stop(&mut self) -> bool {
        self.signal_to_stop() && self.wait_for_thread_to_stop()
    }
    
    /// Spawns a background thread that caches the video.
    pub fn cache(&mut self) -> bool {
        if self.is_still_alive() { return false; };
        let video_data = self.data.clone();
        self.thread = Some(thread::spawn(move || {
            let sleep_duration = Duration::from_millis(250);
            // 'outer indicates that breaking from this loop will immedeately end the thread.
            'outer: loop {
                let (width, height, vid_arc, commands) = { // minimize the duration of the lock on video_data (self.data @ VideoWithAutoCache)
                    let mut data = video_data.lock().unwrap();
                    let mut commands = Vec::with_capacity(data.commands.len());
                    loop {
                        match data.commands.pop() {
                            Some(v) => commands.push(v),
                            None => break,
                        };
                    };
                    (data.width, data.height, data.vid.clone(), commands)
                };
                for command in commands.into_iter() {
                    match command {
                        BackgroundThreadCommand::Stop => break 'outer,
                    }
                }
                if width != 0 && height != 0 {
                    let optimal_caching_index = {
                        let last_draw = {
                            let mut vid = vid_arc.lock().unwrap();
                            vid.last_draw.with_resolution(width, height)
                        }; // lock on vid is dropped again
                        if let Some(last_draw) = last_draw {
                            last_draw.lock().unwrap().get_most_useful_index_for_caching() // lock on last_draw is dropped again
                        } else {
                            1.0
                        }
                    };
                    {
                        let mut vid = vid_arc.lock().unwrap(); // lock the mutex of the actual video while - THIS IS NOT WHAT WE WANT TO DO
                        if let Some(prep_data) = vid.prep_draw(optimal_caching_index) {
                            vid.draw(&mut image::DynamicImage::new_rgba8(width, height), prep_data, &crate::video_render_settings::VideoRenderSettings::perfect_but_slow());
                        } else {
                        };
                    };
                };
                thread::sleep(sleep_duration);
            };
        }));
        true
    }
}