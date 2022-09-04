use speedy2d::{dimen::Vector2, color::Color, image::{ImageDataType, ImageSmoothingMode}, shape::Rectangle};

use crate::{multithreading::{automatically_cache_frames::VideoWithAutoCache}, video_cached_frames::VideoCachedFramesOfCertainResolution, video::{VideoChanges, VideoTypeChanges, VideoTypeChanges_List}, curve::Curve, content::content::Content, gui::speedy2d::{layout::{CustomDrawActions, InputAction, EditorWindowLayoutContentData}, content_list::EditorWindowLayoutContentEnum}};

use super::super::layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode};

pub struct VideoPreview {
    time_created: std::time::Instant,
    video: VideoWithAutoCache,
    vid_cached_frames: VideoCachedFramesOfCertainResolution,
    progress: f64,
    mouse_on_progress_bar: Option<f64>,
    mouse_left_button_down_started_on_progress_bar: bool,
    layout_content_data: EditorWindowLayoutContentData,
}
impl VideoPreview {
    pub fn new(video: VideoWithAutoCache) -> Self {
        let vid_cached_frames = Self::get_with_resulution(&video, 0, 0);
        Self { time_created: std::time::Instant::now(), video, vid_cached_frames, progress: 0.0, mouse_on_progress_bar: None, mouse_left_button_down_started_on_progress_bar: false, layout_content_data: EditorWindowLayoutContentData::default(), }
    }
}
impl VideoPreview {
    fn get_with_resulution(video: &VideoWithAutoCache, width: u32, height: u32) -> VideoCachedFramesOfCertainResolution {
        let vid = video.get_vid_mutex_arc();
        let mut vid = vid.lock().unwrap();
        vid.last_draw.with_resolution_or_create(width, height)
    }
    /// Change the resolution of the preview
    fn update_with_resolution(&mut self, width: u32, height: u32) {
        self.video.set_width_and_height(width, height);
        self.vid_cached_frames = Self::get_with_resulution(&self.video, width, height);
    }
}
impl VideoPreview {
    fn draw_type_preview(&mut self, moving /* 0.0 = no, 1.0 = yes */: f32, visibility: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32)) {
        let line_color = Color::from_rgba(1.0 - moving, 1.0 - moving, 1.0, visibility);
        let left = position.0 + 0.5;
        let right = position.0 + position.2 - 0.5;
        let top = position.1 + 0.5;
        let bottom = position.1 + position.3 - 0.5;
        let middle = position.0 + 0.5 * position.2;
        let center = position.1 + 0.5 * position.3;
        // Triangle (to indicate video)
        let tri_left = position.0 + position.2 * 0.4;
        let tri_right = position.0 + position.2 * 0.6;
        let tri_top = position.1 + position.3 * 0.4;
        let tri_bottom = position.1 + position.3 * 0.6;
        let tri_color = Color::from_rgba(1.0 - moving * 0.3, 1.0 - moving * 0.6, 1.0 - moving * 0.6, visibility);
        graphics.draw_line(
            Vector2::new(tri_left, tri_top),
            Vector2::new(tri_left, tri_bottom),
            2.0, tri_color,
        );
        graphics.draw_line(
            Vector2::new(tri_left, tri_top),
            Vector2::new(tri_right, center),
            2.0, tri_color,
        );
        graphics.draw_line(
            Vector2::new(tri_left, tri_bottom),
            Vector2::new(tri_right, center),
            2.0, tri_color,
        );
        // Outline (rectangle)
        //  --
        graphics.draw_line(
            Vector2::new(left, top),
            Vector2::new(right, top),
            1.0, line_color,
        );
        // |
        graphics.draw_line(
            Vector2::new(left, top),
            Vector2::new(left, bottom),
            1.0, line_color,
        );
        // __
        graphics.draw_line(
            Vector2::new(left, bottom),
            Vector2::new(right, bottom),
            1.0, line_color,
        );
        //  |
        graphics.draw_line(
            Vector2::new(right, top),
            Vector2::new(right, bottom),
            1.0, line_color,
        );
    }
    /// Visibility: 1.0 = normal, 0.0 = invisible, smoothly going from 1.0 to 0.0 => fade out.
    fn draw_type_normal(&mut self, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), visibility: f32) {
    {
        let img = self.vid_cached_frames.cache().lock().unwrap();
        let img = img.get_frame(match self.mouse_on_progress_bar { Some(v) => v, None => self.progress, });
        if let Some((_, img)) = img {
            let handle = graphics.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(img.frame.width(), img.frame.height()), img.frame.as_bytes()).unwrap();
            graphics.draw_rectangle_image_tinted(Rectangle::new(Vector2::new(position.0, position.1), Vector2::new(position.0 + position.2, position.1 + position.3 * 0.95)), Color::from_rgba(1.0, 1.0, 1.0, visibility), &handle);
        };
    };

    let progress_bar_width = 0.8 * {
        let invisibility = 1.0 - visibility;
        1.0 - invisibility * invisibility * invisibility
    };

    let progress_bar_space_on_side = (1.0 - progress_bar_width) / 2.0 * position.2;
    let progress_bar_width = progress_bar_width * position.2;
    let progress_bar_line_y = position.1 + position.3 * 0.97;
    let progress_bar_pos_x = position.0 + progress_bar_space_on_side + progress_bar_width * self.progress as f32;
    graphics.draw_line(Vector2::new(position.0 + progress_bar_space_on_side, progress_bar_line_y), Vector2::new(progress_bar_pos_x, progress_bar_line_y), 1.0, Color::CYAN);
    graphics.draw_line(Vector2::new(progress_bar_pos_x, progress_bar_line_y), Vector2::new(position.0 + progress_bar_space_on_side + progress_bar_width, progress_bar_line_y), 1.0, Color::BLUE);}
}
impl EditorWindowLayoutContentTrait for VideoPreview {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let video_render_size = (320, 180);//(position.2.floor() as u32, (position.3 * 0.95).floor() as u32);

        {
            // ok(n) will return true the first time, then always false. Useful to ensure a certain action is only executed once even if it is present multiple times.
            let mut ok = [false; 2]; let mut ok = |i: usize| !std::mem::replace(&mut ok[i], true);
            //
            for action in input.get_custom_actions().unwrap().iter() {
                match action {
                    CustomDrawActions::VideoPreviewResize(false) => if ok(0) { self.update_with_resolution(0, 0); input.add_custom_action(CustomDrawActions::VideoPreviewResize(true)); },
                    CustomDrawActions::VideoPreviewResize(true) => if ok(1) { self.update_with_resolution(video_render_size.0, video_render_size.1); },
                    CustomDrawActions::SetEditingTo(_) => (), // TODO?
                };
            };
        };

        if draw_opts.force_redraw_due_to_resize {
            self.update_with_resolution(video_render_size.0, video_render_size.1);
        };
        match &draw_opts.draw_mode {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    self.draw_type_normal(graphics, position, 1.0);
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, 1.0, graphics, position)
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => {
                match modes {
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => {
                        self.draw_type_normal(graphics, position, 1.0);
                    },
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                        self.draw_type_normal(graphics, position, 1.0 - prog);
                        self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, *prog, graphics, position);
                    },
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                        self.draw_type_normal(graphics, position, *prog);
                        self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, 1.0 - prog, graphics, position);
                    },
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old, }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new, }] => {
                        if *moving_old == *moving_new {
                            self.draw_type_preview(if *moving_new { 1.0 } else { 0.0 }, 1.0, graphics, position)
                        } else if *moving_new {
                            self.draw_type_preview(*prog, 1.0, graphics, position)
                        } else {
                            self.draw_type_preview(1.0 - prog, 1.0, graphics, position)
                        }
                    },
                }
            },
        }
    }

    fn handle_input_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            InputAction::None => (),
            InputAction::Mouse(action) => match action {
                crate::gui::speedy2d::layout::MouseAction::Moved => {
                    let mouse_pos = input.clonable.mouse_pos;
                    fn on_progress_bar(mouse_pos: (f32, f32)) -> bool {
                        (mouse_pos.1 - 0.97).abs() <= 0.02 && mouse_pos.0 >= 0.05 && mouse_pos.0 <= 0.95
                    }
                    let mouse_held_from_progress_bar = self.mouse_left_button_down_started_on_progress_bar && input.owned.mouse_down_buttons.contains_key(&speedy2d::window::MouseButton::Left);
                    self.mouse_on_progress_bar = if mouse_held_from_progress_bar || on_progress_bar(mouse_pos) {
                        let prog = ((mouse_pos.0 as f64 - 0.1) / 0.8).max(0.0).min(1.0);
                        if mouse_held_from_progress_bar { self.progress = prog; };
                        Some(prog)
                    } else { None };
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => {
                    self.mouse_left_button_down_started_on_progress_bar = false;
                    match self.mouse_on_progress_bar {
                        Some(prog) => match btn {
                            speedy2d::window::MouseButton::Left => {self.progress = prog; self.mouse_left_button_down_started_on_progress_bar = true; }
                            _ => (),
                        },
                        None => {
                            let mouse_pos = input.clonable.mouse_pos;
                            if 0.0 <= mouse_pos.0 && mouse_pos.0 <= 1.0 && 0.0 <= mouse_pos.1 && mouse_pos.1 <= 1.0 {
                            };
                        },
                    };
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonUp(_) => (),
                crate::gui::speedy2d::layout::MouseAction::Scroll(_) => (),
            },
            InputAction::Keyboard(action) => {},
        }
    }
    
    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::VideoPreview(self).into()
    }

    fn as_window_title(&self) -> String {
        format!("video preview")
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(&mut self) -> &mut [crate::gui::speedy2d::content_list::EditorWindowLayoutContent] {
        &mut []
    }
}