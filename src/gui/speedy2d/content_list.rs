use super::{content, layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawOptions, UserInput, EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentDrawMode, InputAction, MouseAction, EditorWindowLayoutContentData}, request::EditorWindowLayoutRequest};
use speedy2d::{Graphics2D, dimen::Vector2, color::Color, font::{TextLayout, TextOptions}};

pub struct EditorWindowLayoutContent {
    pub c: EditorWindowLayoutContentEnum,
}
pub enum EditorWindowLayoutContentEnum {
    Placeholder(content::placeholder::Placeholder),
    VideoPreview(content::video_preview::VideoPreview),
    VideoTree(content::video_tree::VideoTree),
    VideoPropertiesEditor(content::video_properties_editor::VideoPropertiesEditor),

    LayoutHalf(Box<content::layout::half::Half>),

    SpecialQVidRunner(content::special::qvidrunner::QVidRunner),
}

impl EditorWindowLayoutContent {
    pub fn replace(&mut self, new: Self) -> Self {
        std::mem::replace(self, new)
    }
    pub fn take(&mut self) -> Self {
        self.replace(content::placeholder::Placeholder::new().as_enum())
    }

    pub fn height_of_top_bar_in_type_preview_mode_respecting_draw_mode(&self, draw_opts: &EditorWindowLayoutContentDrawOptions, in_pixels: bool) -> f32 {
        let height_of_top_bar_in_type_preview_mode = if in_pixels { self.height_of_top_bar_in_type_preview_mode_in_pixels(draw_opts) } else { self.height_of_top_bar_in_type_preview_mode() };
        match &draw_opts.draw_mode {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    0.0
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    height_of_top_bar_in_type_preview_mode
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => match modes {
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old, }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new, }] => {
                    0.0
                },
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                    prog * height_of_top_bar_in_type_preview_mode
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                    (1.0 - prog) * height_of_top_bar_in_type_preview_mode
                },
                _ => {
                    0.0
                },
            },
        }
    }
    pub fn height_of_top_bar_in_type_preview_mode_in_pixels(&self, draw_opts: &EditorWindowLayoutContentDrawOptions) -> f32 {
        self.height_of_top_bar_in_type_preview_mode() * draw_opts.render_canvas_size.1 as f32
    }
    pub fn height_of_top_bar_in_type_preview_mode(&self) -> f32 {
        match &self.c {
            EditorWindowLayoutContentEnum::VideoPreview(_) |
            EditorWindowLayoutContentEnum::VideoTree(_) |
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(_) => 0.05,
            EditorWindowLayoutContentEnum::Placeholder(_) |
            EditorWindowLayoutContentEnum::LayoutHalf(_) |
            EditorWindowLayoutContentEnum::SpecialQVidRunner(_) => 0.025
        }
    }
    pub fn height_relative_from_pixels(draw_opts: &EditorWindowLayoutContentDrawOptions, height: f32) -> f32 { height / draw_opts.my_size_in_pixels.1 }
}

impl EditorWindowLayoutContent {
    pub fn was_changed(&self) -> bool { self.was_changed_custom() }
    pub fn draw_onto(&mut self, draw_opts: &mut EditorWindowLayoutContentDrawOptions, graphics: &mut Graphics2D, position: &(f32, f32, f32, f32), input: &mut UserInput) {
        let height_of_top_bar_in_type_preview_mode = self.height_of_top_bar_in_type_preview_mode_in_pixels(draw_opts);
        let height_of_top_bar = self.height_of_top_bar_in_type_preview_mode_respecting_draw_mode(draw_opts, true);
        let new_position = (position.0, position.1 + height_of_top_bar, position.2, (position.3 - height_of_top_bar).max(0.0));
        
        /* window title drawing */ {
            let draw_window_title = match &draw_opts.draw_mode {
                EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                    EditorWindowLayoutContentSDrawMode::Normal => None,
                    EditorWindowLayoutContentSDrawMode::TypePreview { moving, } => Some(if *moving { 0.7 } else { 1.0 }),
                },
                EditorWindowLayoutContentDrawMode::Transition { modes, prog } => match modes {
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => None,
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => Some(if *moving { 0.7 * prog } else { *prog }),
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => Some(if *moving { 0.7 * (1.0 - prog) } else { 1.0 - prog }),
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new }] => Some(match (moving_old, moving_new) {
                        (true, true) => 0.7,
                        (false, false) => 1.0,
                        (true, false) => 0.7 + 0.3 * prog,
                        (false, true) => 1.0 - 0.3 * prog,
                    })
                },
            };
            if let Some(mut draw_window_title) = draw_window_title {
                if position.3 < height_of_top_bar { draw_window_title *= position.3 / height_of_top_bar; };
                let text = draw_opts.assets_manager.get_default_font().layout_text(self.as_window_title().as_str(), height_of_top_bar_in_type_preview_mode * 0.8, TextOptions::new());
                graphics.draw_text(Vector2::new(position.0, position.1), Color::from_rgba(1.0, 1.0, 1.0, draw_window_title), &text)
            };
        };
        if new_position.2 > 0.0 && new_position.3 > 0.0 {
            self.draw_onto_custom(draw_opts, graphics, &new_position, input);
        }
    }
    pub fn handle_input(&mut self, draw_opts: &mut EditorWindowLayoutContentDrawOptions, input: &mut UserInput) {
        // handle input on top bar
        let top_bar_height_in_px = self.height_of_top_bar_in_type_preview_mode_respecting_draw_mode(draw_opts, true);
        let top_bar_height_relative = Self::height_relative_from_pixels(draw_opts, top_bar_height_in_px);
        match &draw_opts.draw_mode {
            EditorWindowLayoutContentDrawMode::Static(EditorWindowLayoutContentSDrawMode::TypePreview { moving: false, }) => {
                match &input.owned.action {
                    InputAction::None | InputAction::Keyboard(_) => (),
                    InputAction::Mouse(action) => match action {
                        MouseAction::ButtonDown(btn) => match btn {
                            speedy2d::window::MouseButton::Left => {
                                if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < top_bar_height_relative {
                                    self.data().requests.push(EditorWindowLayoutRequest::TypePreviewModeBecomeDraggedWindowStart { size: draw_opts.my_size_in_pixels.clone(), grab_position: input.clonable.mouse_pos.clone(), });
                                };
                            },
                            _ => (),
                        },
                        MouseAction::ButtonUp(btn) => match btn {
                            speedy2d::window::MouseButton::Left => {
                                if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
                                    self.data().requests.push(EditorWindowLayoutRequest::TypePreviewModeBecomeDraggedWindowEnd);
                                };
                            },
                            _ => (),
                        },
                        _ => (),
                    },
                };
            },
            _ => (),
        };

        input.clonable.mouse_pos.1 -= top_bar_height_relative;
        input.clonable.mouse_pos.1 /= 1.0 - top_bar_height_relative;
        self.handle_input_custom(draw_opts, input)
    }
}

impl From<EditorWindowLayoutContentEnum> for EditorWindowLayoutContent { fn from(v: EditorWindowLayoutContentEnum) -> Self { Self { c: v, } } }

impl EditorWindowLayoutContentTrait for EditorWindowLayoutContent {
    fn was_changed_custom(&self) -> bool {
        match &self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.was_changed_custom(),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.was_changed_custom(),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.was_changed_custom(),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.was_changed_custom(),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.was_changed_custom(),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.was_changed_custom(),
        }
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut super::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut super::layout::UserInput) {
        match &mut self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.draw_onto_custom(draw_opts, graphics, position, input),
        }
    }

    fn handle_input_custom(&mut self, draw_opts: &mut super::layout::EditorWindowLayoutContentDrawOptions, input: &mut super::layout::UserInput) {
        match &mut self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.handle_input_custom(draw_opts, input),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.handle_input_custom(draw_opts, input),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.handle_input_custom(draw_opts, input),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.handle_input_custom(draw_opts, input),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.handle_input_custom(draw_opts, input),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.handle_input_custom(draw_opts, input),
        }
    }
    
    fn as_enum(self) -> self::EditorWindowLayoutContent {
        self
    }

    fn as_window_title(&self) -> String {
        match &self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.as_window_title(),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.as_window_title(),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.as_window_title(),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.as_window_title(),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.as_window_title(),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.as_window_title(),
        }
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData {
        match &mut self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.data(),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.data(),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.data(),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.data(),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.data(),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.data(),
        }
    }

    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent] {
        match &mut self.c {
            EditorWindowLayoutContentEnum::Placeholder(v) => v.get_children(),
            EditorWindowLayoutContentEnum::VideoPreview(v) => v.get_children(),
            EditorWindowLayoutContentEnum::VideoTree(v) => v.get_children(),
            EditorWindowLayoutContentEnum::VideoPropertiesEditor(v) => v.get_children(),
            EditorWindowLayoutContentEnum::LayoutHalf(v) => v.get_children(),
            EditorWindowLayoutContentEnum::SpecialQVidRunner(v) => v.get_children(),
        }
    }
}