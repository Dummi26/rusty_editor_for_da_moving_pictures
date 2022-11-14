use image::imageops::FilterType;

pub struct VideoRenderSettings {
    /// The distance (in frames) that the displayed frame can have from the desired frame. This might become useless once actual good video loading is implemented.
    pub max_distance_when_retrieving_closest_frame: i8,
    /// How to up- or downscale images. Very likely to have a big performance impact.
    pub image_scaling_filter_type: FilterType,
    pub this_frame: FrameRenderInfo,
}
impl VideoRenderSettings {
    /// This is mostly used for the preview.
    pub fn preview() -> Self { Self {
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
        this_frame: Default::default(),
    } }
    /// This is used for final render. It prevents inaccuracies.
    pub fn export() -> Self { Self {
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
        this_frame: Default::default(),
    } }
    pub fn new_frame(&mut self) {
        self.this_frame = Default::default();
    }
}

pub struct FrameRenderInfo {
    pub my_size: (f64, f64),
}
impl Default for FrameRenderInfo {
    fn default() -> Self { Self {
        my_size: (1.0, 1.0),
    } }
}