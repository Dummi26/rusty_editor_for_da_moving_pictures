use speedy2d::{window::VirtualKeyCode, dimen::Vector2, color::Color, font::{TextOptions, TextLayout}};

use crate::{gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentData}, content_list::EditorWindowLayoutContentEnum}, useful};

pub struct QVidRunner {
    pub query: String,
    layout_content_data: EditorWindowLayoutContentData,
}

impl EditorWindowLayoutContentTrait for QVidRunner {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let text = draw_opts.assets_manager.get_default_font().layout_text(self.query.as_str(), position.3 * 0.75, TextOptions::new());
        let rx = (position.2 - text.width()) / 2.0;
        graphics.draw_text(Vector2::new(position.0 + rx, position.1),
            Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            &text
        );
    }

    fn handle_input_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None |
            crate::gui::speedy2d::layout::InputAction::Mouse(_) => (),
            crate::gui::speedy2d::layout::InputAction::Keyboard(action) => match action {
                crate::gui::speedy2d::layout::KeyboardAction::Pressed(_, _) |
                crate::gui::speedy2d::layout::KeyboardAction::Released(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Typed(ch) => {
                    if !(input.owned.keyboard_modifiers_state.logo() || input.owned.keyboard_modifiers_state.ctrl() || input.owned.keyboard_modifiers_state.alt()) {
                        match useful::CharOrAction::from(ch) {
                            useful::CharOrAction::Char(ch) => self.query.push(ch.clone()),
                            useful::CharOrAction::Enter => self.query.clear(),
                            useful::CharOrAction::Delete |
                            useful::CharOrAction::Backspace => self.query = { let mut vec = Vec::from_iter(self.query.chars()); vec.pop(); String::from_iter(vec.into_iter()) },
                            useful::CharOrAction::Tab => (),
                            useful::CharOrAction::Ignored => (),
                        };
                    };
                },
            },
        };
    }

    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::SpecialQVidRunner(self).into()
    }
    fn as_enum_type(&self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::SpecialQVidRunner
    }

    fn as_window_title(&self) -> String {
        "QVidRunner".to_string()
    }

    fn data(&mut self) -> &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(&mut self) -> &mut [crate::gui::speedy2d::content_list::EditorWindowLayoutContent] {
        &mut []
    }
}

impl QVidRunner {
    pub fn new() -> Self {
        Self { query: String::new(), layout_content_data: EditorWindowLayoutContentData::default(), }
    }
}
