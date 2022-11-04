use image::imageops::FilterType;

pub struct VideoRenderSettings {
    /// The distance (in frames) that the displayed frame can have from the desired frame. This might become useless once actual good video loading is implemented.
    pub max_distance_when_retrieving_closest_frame: i8,
    /// How to up- or downscale images. Very likely to have a big performance impact.
    pub image_scaling_filter_type: FilterType,
}
impl VideoRenderSettings {
    /// This is mostly used for the preview.
    pub fn preview() -> Self { Self {
        max_distance_when_retrieving_closest_frame: 120,
        image_scaling_filter_type: FilterType::Nearest,
    } }
    /// This is used for final render. It prevents inaccuracies.
    pub fn export() -> Self { Self {
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
    } }
}