use image::imageops::FilterType;

pub struct VideoRenderSettings {
    pub allow_retrieval_of_cached_frames: Option<f64>,
    pub max_distance_when_retrieving_closest_frame: i8,
    pub image_scaling_filter_type: FilterType,
}
impl VideoRenderSettings {
    /// This is mostly used for the preview. It allows caching (100 frames per object) and allows retrieval of "closest" frames that are very far away.
    pub fn preview() -> Self { Self {
        allow_retrieval_of_cached_frames: Some(1.0),
        max_distance_when_retrieving_closest_frame: 120,
        image_scaling_filter_type: FilterType::Nearest,
    } }
    /// Uses the low quality settings from Self::preview(), but does not allow frames to deviate from the user's selected time.
    pub fn exact_timing_preview() -> Self { Self {
        allow_retrieval_of_cached_frames: Some(0.0),
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Nearest,
    }}
    /// This is used for the 'exact' slider. It prevents inaccuracies, but it uses caching so it doesn't need to redraw every frame from scratch.
    pub fn perfect_with_caching() -> Self { Self {
        allow_retrieval_of_cached_frames: Some(0.0),
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
    }}
    /// This is used for final render. It prevents any and all inaccuracies and even disables caching.
    pub fn perfect_but_slow() -> Self { Self {
        allow_retrieval_of_cached_frames: None,
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
    } }
    pub fn caching_thread() -> Self { Self::exact_timing_preview() }
}