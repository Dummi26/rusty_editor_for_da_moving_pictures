use speedy2d::{font::TextLayout, dimen::Vector2, color::Color};

use crate::gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentData}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum, EditorWindowLayoutContentTypeEnum}};

#[derive(Default)]
pub struct Placeholder {
    layout_content_data: EditorWindowLayoutContentData,
    types_which_can_replace_hovered: [HoveredStatus; Self::TYPES_WHICH_CAN_REPLACE_COUNT],
    mouse_down_selected: Option<usize>,
}
impl Placeholder { pub fn new() -> Self {
    Self { ..Default::default() }
}}
impl EditorWindowLayoutContentTrait for Placeholder {
    fn was_changed_custom(&self) -> bool {
        false
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), _input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &draw_opts.draw_mode.clone() /* TODO: Somehow don't clon? (same problem in tree view) */ {
            crate::gui::speedy2d::EditorWindowLayoutContentDrawMode::Static(mode) => {
                match mode {
                    crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::Normal => self.draw_normal(1.0, graphics, position),
                    crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving } => self.draw_type_preview(1.0, if *moving { 1.0 } else { 0.0 }, draw_opts, graphics, position),
                }
            },
            crate::gui::speedy2d::EditorWindowLayoutContentDrawMode::Transition { modes, prog, } => {
                match modes {
                    [crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::Normal, crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::Normal] => self.draw_normal(1.0, graphics, position),
                    [crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::Normal, crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                        self.draw_normal(1.0 - prog, graphics, position);
                        self.draw_type_preview(*prog, if *moving { 1.0 } else { 0.0 }, draw_opts, graphics, position);
                    },
                    [crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving }, crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::Normal] => {
                        self.draw_type_preview(1.0 - prog, if *moving { 1.0 } else { 0.0 }, draw_opts, graphics, position);
                        self.draw_normal(*prog, graphics, position);
                    },
                    [crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old }, crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new }] => {
                        self.draw_type_preview(1.0, if !moving_old && *moving_new { *prog } else if *moving_old && !moving_new { 1.0 - prog } else if *moving_old && *moving_new { 1.0 } else { 0.0 }, draw_opts, graphics, position);
                    },
                }
            },
        };
    }

    fn handle_input_custom(&mut self, _draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        let mouse_currently_on = if input.clonable.mouse_pos.0 > 0.0 && input.clonable.mouse_pos.0 < 1.0 && input.clonable.mouse_pos.1 > 0.0 && input.clonable.mouse_pos.1 < 1.0 {
            Some((input.clonable.mouse_pos.1 * Self::TYPES_WHICH_CAN_REPLACE_COUNT as f32).floor() as usize)
        } else { None };

        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                crate::gui::speedy2d::layout::MouseAction::Moved => {
                    // Set everything to not hovered except for the one that the mouse is on
                    let mut now: Option<std::time::Instant> = None;
                    for (i, hov) in self.types_which_can_replace_hovered.iter_mut().enumerate() {
                        if Some(i) == mouse_currently_on {
                            match hov {
                                HoveredStatus::NotHovered => {
                                    let now = if let Some(now) = now { now.clone() } else { let n = std::time::Instant::now(); now = Some(n); n.clone() }; // only request time from system once, or not at all if it's not necessary
                                    *hov = HoveredStatus::Starting(now);
                                },
                                HoveredStatus::Starting(_) => (),
                                HoveredStatus::Hovered => (),
                                HoveredStatus::Ending(t) => {
                                    let now = if let Some(now) = now { now.clone() } else { let n = std::time::Instant::now(); now = Some(n); n.clone() }; // only request time from system once, or not at all if it's not necessary
                                    *hov = HoveredStatus::Starting(now + t.elapsed() - std::time::Duration::from_secs_f32(0.25));
                                },
                            };
                        } else {
                            match hov {
                                HoveredStatus::NotHovered => (),
                                HoveredStatus::Starting(t) => {
                                    let now = if let Some(now) = now { now.clone() } else { let n = std::time::Instant::now(); now = Some(n); n.clone() }; // only request time from system once, or not at all if it's not necessary
                                    *hov = HoveredStatus::Ending(now + t.elapsed() - std::time::Duration::from_secs_f32(0.25));
                                },
                                HoveredStatus::Hovered => {
                                    let now = if let Some(now) = now { now.clone() } else { let n = std::time::Instant::now(); now = Some(n); n.clone() }; // only request time from system once, or not at all if it's not necessary
                                    *hov = HoveredStatus::Ending(now);
                                },
                                HoveredStatus::Ending(_) => (),
                            };
                        };
                    };
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => match btn {
                    speedy2d::window::MouseButton::Left => {
                        self.mouse_down_selected = mouse_currently_on;
                    },
                    speedy2d::window::MouseButton::Middle => (),
                    speedy2d::window::MouseButton::Right => (),
                    speedy2d::window::MouseButton::Other(_) => (),
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonUp(btn) => match btn {
                    speedy2d::window::MouseButton::Left => {
                        if let Some(mouse_currently_on) = mouse_currently_on {
                            if let Some(mouse_down_on) = self.mouse_down_selected {
                                if mouse_currently_on == mouse_down_on {
                                    self.replace_self(mouse_currently_on);
                                };
                            };
                        };
                        self.mouse_down_selected = None;
                    },
                    speedy2d::window::MouseButton::Middle => (),
                    speedy2d::window::MouseButton::Right => (),
                    speedy2d::window::MouseButton::Other(_) => (),
                },
                crate::gui::speedy2d::layout::MouseAction::Scroll(_) => (),
            },
            crate::gui::speedy2d::layout::InputAction::Keyboard(action) => match action {
                crate::gui::speedy2d::layout::KeyboardAction::Pressed(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Released(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Typed(_) => (),
            },
        }
    }

    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::Placeholder(self).into()
    }
    fn as_enum_type(&self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::Placeholder
    }

    fn as_window_title(&self) -> String {
        format!("placeholder")
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent] {
        &mut []
    }
}

impl Placeholder {
    fn draw_normal(&mut self, shown: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32)) {
        graphics.draw_line(
            Vector2::new(position.0 as f32 + 0.25 * position.2 as f32, position.1 as f32 + 0.5 * position.3 as f32),
            Vector2::new(position.0 as f32 + 0.75 * position.2 as f32, position.1 as f32 + 0.5 * position.3 as f32),
            1.0, Color::from_rgba(1.0, 1.0, 1.0, shown)
        );
    }

    const TYPES_WHICH_CAN_REPLACE_COUNT: usize = 3;
    const TYPES_WHICH_CAN_REPLACE_NAMES: [&'static str; Self::TYPES_WHICH_CAN_REPLACE_COUNT] = [
        "Split",
        "Tree View",
        "Properties",
    ];

    fn draw_type_preview(&mut self, shown: f32, _moving: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32)) {
        let elem_height = position.3 / Self::TYPES_WHICH_CAN_REPLACE_COUNT as f32;
        for (elem, name) in Self::TYPES_WHICH_CAN_REPLACE_NAMES.iter().enumerate() {
            let text = draw_opts.assets_manager.get_default_font().layout_text(name, elem_height * 0.8, speedy2d::font::TextOptions::new());
            let hovered_status = &mut self.types_which_can_replace_hovered[elem];
            let (r, g, b) = match hovered_status {
                HoveredStatus::NotHovered => (0.5, 0.5, 0.5),
                HoveredStatus::Starting(t) => {
                    let t = (t.elapsed().as_secs_f32() / 0.25).min(1.0);
                    let o = (0.5, 0.5 + 0.5 * t, 0.5);
                    if t == 1.0 { *hovered_status = HoveredStatus::Hovered; };
                    o
                },
                HoveredStatus::Hovered => (0.5, 1.0, 0.5),
                HoveredStatus::Ending(t) => {
                    let t = (t.elapsed().as_secs_f32() / 0.25).min(1.0);
                    let o = (0.5, 1.0 - 0.5 * t, 0.5);
                    if t == 1.0 { *hovered_status = HoveredStatus::NotHovered; };
                    o
                },
            };
            graphics.draw_text(Vector2 { x: position.0 + (position.2 - text.width()) / 2.0, y: position.1 + elem as f32 * position.3 / Self::TYPES_WHICH_CAN_REPLACE_COUNT as f32 }, Color::from_rgba(r, g, b, shown), &text);
        };
    }
    fn replace_self(&mut self, with_what: usize) {
        match EditorWindowLayoutContentTypeEnum::Placeholder { // this does nothing, but forces the programmer to at least consider adding any new content type to the list (because every time the enum grows, this code will error).
            EditorWindowLayoutContentTypeEnum::Placeholder => (),
            EditorWindowLayoutContentTypeEnum::VideoPreview => (),
            EditorWindowLayoutContentTypeEnum::VideoTree => (),
            EditorWindowLayoutContentTypeEnum::VideoPropertiesEditor => (),
            EditorWindowLayoutContentTypeEnum::LayoutHalf => (),
            EditorWindowLayoutContentTypeEnum::SpecialQVidRunner => (),
        };

        match match with_what {
            0 => Some(crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::LayoutHalf),
            1 => Some(crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::VideoTree),
            2 => Some(crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::VideoPropertiesEditor),
            _ => None,
        } {
            None => (),
            Some(v) => self.data().requests.push(crate::gui::speedy2d::request::EditorWindowLayoutRequest::ChangeMeTo(v)),
        };
    }
}

enum HoveredStatus {
    NotHovered,
    Starting(std::time::Instant),
    Hovered,
    Ending(std::time::Instant),
}
impl Default for HoveredStatus {
    fn default() -> Self { Self::NotHovered }
}
