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
    pub fn preview(this_frame: FrameRenderInfo) -> Self { Self {
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
        this_frame,
    } }
    /// This is used for final render. It prevents inaccuracies.
    pub fn export(this_frame: FrameRenderInfo) -> Self { Self {
        max_distance_when_retrieving_closest_frame: 0,
        image_scaling_filter_type: FilterType::Gaussian,
        this_frame,
    } }
}

pub struct FrameRenderInfo {
    pub out_vid_aspect_ratio: f64,
}
impl FrameRenderInfo {
    pub fn new(out_vid_aspect_ratio: f64) -> Self { Self {
        out_vid_aspect_ratio,
    } }
}