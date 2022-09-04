use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use image::DynamicImage;

pub struct VideoCachedFrames {
    resolutions: Vec<VideoCachedFramesOfCertainResolution>,
    max_resolutions: usize,
}
impl VideoCachedFrames {
    pub fn new() -> Self {
        Self {
            resolutions: Vec::new(),
            max_resolutions: 1,
        }
    }
    
    pub fn with_resolution(&self, width: u32, height: u32) -> Option<VideoCachedFramesOfCertainResolution> {
        for res in self.resolutions.iter() {
            if res.width == width && res.height == height {
                return Some(res.clone());
            };
        };
        None
    }
    pub fn with_resolution_or_create(&mut self, width: u32, height: u32) -> VideoCachedFramesOfCertainResolution {
        for res in self.resolutions.iter() {
            if res.width == width && res.height == height {
                return res.clone();
            };
        };
        let new = VideoCachedFramesOfCertainResolution::new(width, height);
        self.add_resolution(new.clone());
        new
    }
    
    fn add_resolution(&mut self, res: VideoCachedFramesOfCertainResolution) {
        if self.resolutions.len() >= self.max_resolutions { self.resolutions.pop(); };
        self.resolutions.insert(0, res);
    }
    
    pub fn clear_resolutions(&mut self) {
        for res in &mut self.resolutions {
            res.clear();
        };
        self.resolutions.clear();
    }
    
    pub fn add_frame(&mut self, progress: f64, width: u32, height: u32, frame: DynamicImage) {
        self.with_resolution_or_create(width, height).cache().lock().unwrap().add_frame(progress, frame);
    }
    pub fn add_frames<T>(&mut self, width: u32, height: u32, prog_and_frames: T) where T: IntoIterator<Item=(f64, DynamicImage)> {
        let res = self.with_resolution_or_create(width, height);
        let mut res = res.cache().lock().unwrap();
        for (progress, frame) in prog_and_frames {
            res.add_frame(progress, frame);
        };
    }
}

#[derive(Clone)]
pub struct VideoCachedFramesOfCertainResolution {
    width: u32,
    height: u32,
    cached_frames: Arc<Mutex<VideoCachedFramesOfCertainResolutionData>>,
}
impl VideoCachedFramesOfCertainResolution {
    pub fn new_frames(width: u32, height: u32, cached_frames: Arc<Mutex<VideoCachedFramesOfCertainResolutionData>>) -> Self {
        Self { width, height, cached_frames, }
    }
    pub fn new_one_frame(width: u32, height: u32, frame: VideoCachedFrame) -> Self {
        Self { width, height, cached_frames: Arc::new(Mutex::new(VideoCachedFramesOfCertainResolutionData::new_one_frame(width, height, frame))), }
    }
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, cached_frames: Arc::new(Mutex::new(VideoCachedFramesOfCertainResolutionData::new(width, height))), }
    }
    
    pub fn clear(&self) {
        Self::clear_from_arc(&self.cached_frames);
    }
    fn clear_from_arc(arc: &Arc<Mutex<VideoCachedFramesOfCertainResolutionData>>) {
        arc.lock().unwrap().cached_frames.clear();
    }
    /// Spawns a background thread that will clear the cache as soon as the mutex locks. Use this to avoid freezing front-end parts of the program.
    pub fn clear_whenever_possible(&self) -> JoinHandle<()> {
        let arc = self.cached_frames.clone();
        thread::spawn(move || {
            Self::clear_from_arc(&arc);
        })
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    
    pub fn cache(&self) -> &Arc<Mutex<VideoCachedFramesOfCertainResolutionData>> { &self.cached_frames }
}
pub struct VideoCachedFramesOfCertainResolutionData {
    width: u32,
    height: u32,
    cached_frames: Vec<VideoCachedFrame>,
}
impl VideoCachedFramesOfCertainResolutionData {
    pub fn new_one_frame(width: u32, height: u32, frame: VideoCachedFrame) -> Self {
        Self { width, height, cached_frames: vec![frame], }
    }
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, cached_frames: Vec::new(), }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    /// If any frames are cached, returns the closest frame and its distance (= (progress-frame.progress).abs())
    pub fn get_frame(&self, progress: f64) -> Option<(f64, &VideoCachedFrame)> {
        let mut mindist = f64::INFINITY;
        let mut minindex = None;
        for (index, frame) in self.cached_frames.iter().enumerate() {
            let dist = (frame.progress - progress).abs();
            if dist < mindist {
                mindist = dist;
                minindex = Some(index);
            };
        };
        if let Some(index) = minindex {
            Some((mindist, &self.cached_frames[index]))
        } else {
            None
        }
    }
    pub fn add_frame(&mut self, progress: f64, frame: DynamicImage) {
        for cached_frame in self.cached_frames.iter_mut() {
            if cached_frame.progress == progress {
                cached_frame.frame = frame;
                return;
            };
        };
        self.cached_frames.insert(0, VideoCachedFrame { progress: progress, frame: frame });
    }

    /// Returns the index and its distance to the nearest cached frame.
    pub fn get_most_useful_index_for_caching(&self) -> (f64, f64) {
        let mut progress = Vec::with_capacity(self.cached_frames.len() + 1);
        for cached_frame in self.cached_frames.iter() {
            progress.push(cached_frame.progress);
        };
        if let Some(last) = progress.last() { if *last != 1.0 { progress.push(1.0); }; };
        let max = u64::MAX as f64; // good enough, probably
        progress.sort_unstable_by_key(|item| (item * max).floor() as u64);
        let mut pval = 0.0;
        let mut best_dist = -2.0;
        let mut optimal_index = 0.0;
        for prog in progress.into_iter() {
            let dist = prog - pval;
            if dist > best_dist {
                best_dist = dist;
                optimal_index = prog - dist / 2.0;
            };
            pval = prog;
        };
        (optimal_index, best_dist.abs())
    }
}

pub struct VideoCachedFrame {
    pub progress: f64,
    pub frame: DynamicImage,
}