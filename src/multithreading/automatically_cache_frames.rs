use std::{thread::{self, JoinHandle}, time::{Duration, Instant}};

use crate::video::Video;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VideoWithAutoCache {
    pub data: Arc<Mutex<VideoWithAutoCacheData>>,
}

/// This is the information the background thread has access to.
/// None of its fields should be operated on for more than a few milliseconds. Anything that takes longer should be behind an Arc<Mutex<..>> so that the main thread, when accessing this struct,
/// does not have to wait.
pub struct VideoWithAutoCacheData {
    vid: Arc<Mutex<Video>>,
    pub commands: Vec<BackgroundThreadCommand>,
    thread: Option<JoinHandle<()>>,
    width: u32,
    height: u32,
}
pub enum BackgroundThreadCommand {
    Stop,
}
impl VideoWithAutoCache {
    pub fn new(vid: Video) -> Self {
        Self { data: Arc::new(Mutex::new(VideoWithAutoCacheData { vid: Arc::new(Mutex::new(vid)), width: 0, height: 0, commands: Vec::new(), thread: None, })), }
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
        let mut data = self.data.lock().unwrap();
        if let Some(thread) = &data.thread {
            if thread.is_finished() {
                data.thread = None;
                false
            } else {
                true
            }
        } else {
            false
        }
    }
    pub fn signal_to_stop(&mut self) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(_) = &data.thread {
            data.commands.push(BackgroundThreadCommand::Stop);
            true
        } else {
            false
        }
    }
    pub fn wait_for_thread_to_stop(&mut self) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(thread) = data.thread.take() {
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
        let mut data = self.data.lock().unwrap();
        data.thread = Some(thread::spawn(move || {
            eprintln!("[bg.c] Starting thread...");
            let sleep_duration = Duration::from_millis(25);
            //
            let mut frames_count = 0u128;
            // 'outer indicates that breaking from this loop will immedeately end the thread.
            'outer: loop {
                let start_time = Instant::now();
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
                            vid.last_draw.with_resolution_or_create(width, height)
                        }; // lock on vid is dropped again
                        let (index, dist) = last_draw.cache().lock().unwrap().get_most_useful_index_for_caching(); // lock on last_draw is dropped again
                        if dist > 0.005 { Some(index) } else { None }
                    };
                    if let Some(optimal_caching_index) = optimal_caching_index {
                        let mut vid = vid_arc.lock().unwrap(); // lock the mutex of the actual video while - THIS IS NOT WHAT WE WANT TO DO
                        if let Some(prep_data) = vid.prep_draw(optimal_caching_index) {
                            vid.draw(&mut image::DynamicImage::ImageRgba8(image::DynamicImage::new_rgb8(width, height).into_rgba8()), prep_data, &crate::video_render_settings::VideoRenderSettings::caching_thread());
                            frames_count += 1;
                            eprintln!("[bg.c] Took {}ms to draw #{}. [{optimal_caching_index}]", start_time.elapsed().as_millis(), frames_count);
                        } else {
                            //eprintln!("[bg.c] no prep_data.");
                        };
                    } else {
                        //eprintln!("[bg.c] no index");
                    };
                };
                //eprintln!("[bg.c] pausing (short)");
                thread::sleep(sleep_duration);
            };
        }));
        true
    }
}