use speedy2d::{dimen::Vector2, color::Color};

use crate::gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentData}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}};

#[derive(Default)]
pub struct Placeholder {
    layout_content_data: EditorWindowLayoutContentData,
}
impl Placeholder { pub fn new() -> Self {
    Self { layout_content_data: EditorWindowLayoutContentData::default() }
}}
impl EditorWindowLayoutContentTrait for Placeholder {
    fn was_changed_custom(&self) -> bool {
        false
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        graphics.draw_line(
            Vector2::new(position.0 as f32 + 0.25 * position.2 as f32, position.1 as f32 + 0.5 * position.3 as f32),
            Vector2::new(position.0 as f32 + 0.75 * position.2 as f32, position.1 as f32 + 0.5 * position.3 as f32),
            1.0, Color::from_rgba(1.0, 1.0, 1.0, 1.0)
        );
    }

    fn handle_input_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
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
