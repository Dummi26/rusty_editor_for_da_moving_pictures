use std::sync::{Arc, Mutex};

use image::DynamicImage;

pub struct VideoCachedFrames {
    resolutions: Vec<VideoCachedFramesOfCertainResolution>,
}
impl VideoCachedFrames {
    pub fn new() -> Self {
        Self {
            resolutions: Vec::new(),
        }
    }
    
    pub fn with_resolution(&mut self, width: u32, height: u32) -> Option<Arc<Mutex<VideoCachedFramesOfCertainResolutionData>>> {
        for res in self.resolutions.iter_mut() {
            if res.width == width && res.height == height {
                return Some(res.cached_frames.clone());
            };
        };
        None
    }
    pub fn with_resolution_or_create(&mut self, width: u32, height: u32) -> Arc<Mutex<VideoCachedFramesOfCertainResolutionData>> {
        for res in self.resolutions.iter_mut() {
            if res.width == width && res.height == height {
                return res.cached_frames.clone();
            };
        };
        let new = Arc::new(Mutex::new(VideoCachedFramesOfCertainResolutionData::new(width, height)));
        self.resolutions.insert(0, VideoCachedFramesOfCertainResolution { width, height, cached_frames: new.clone(), });
        new
    }
    
    pub fn add_frame(&mut self, progress: f64, width: u32, height: u32, frame: DynamicImage) {
        self.with_resolution_or_create(width, height).lock().unwrap().add_frame(progress, frame);
    }
    pub fn add_frames<T>(&mut self, width: u32, height: u32, prog_and_frames: T) where T: IntoIterator<Item=(f64, DynamicImage)> {
        let res = self.with_resolution_or_create(width, height);
        let mut res = res.lock().unwrap();
        for (progress, frame) in prog_and_frames {
            res.add_frame(progress, frame);
        };
    }
}

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

    pub fn get_most_useful_index_for_caching(&self) -> f64 {
        let mut progress = Vec::with_capacity(self.cached_frames.len() + 1);
        for cached_frame in self.cached_frames.iter() {
            progress.push(cached_frame.progress);
        };
        if progress.last() != Some(&1.0) { progress.push(1.0); };
        let max = u64::MAX as f64; // good enough, probably
        progress.sort_unstable_by_key(|item| (item * max).floor() as u64);
        let mut pval = 0.0;
        let mut best_dist = 0.0;
        let mut optimal_index = 1.0;
        for prog in progress.into_iter() {
            let dist = prog - pval;
            if dist > best_dist {
                best_dist = dist;
                optimal_index = prog - dist / 2.0;
            };
            pval = prog;
        };
        optimal_index
    }
}

pub struct VideoCachedFrame {
    pub progress: f64,
    pub frame: DynamicImage,
}