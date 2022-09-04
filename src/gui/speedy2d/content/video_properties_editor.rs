use std::{sync::{Arc, Mutex}, time::Instant};

use speedy2d::{dimen::Vector2, color::Color, font::{TextLayout, TextOptions, TextAlignment}};

use crate::{video::{Video, VideoChanges, VideoTypeEnum}, content::content::Content, gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentData, CustomDrawActions}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}, request::EditorWindowLayoutRequest}, effect, useful};

pub struct VideoPropertiesEditor {
    video: Arc<Mutex<Video>>,
    editing: (Option<(u32, Video, VideoChanges)>, Option<Instant>),
    scroll_dist: f32,
    mouse_pressed_down_on_index: Option<u32>,
    height_of_element: RelOrAbs,
    layout_content_data: EditorWindowLayoutContentData,
} enum RelOrAbs { Rel(f32), Abs(f32), }
impl VideoPropertiesEditor {
    pub fn new(video: Arc<Mutex<Video>>) -> Self {
        Self {
            video,
            editing: (None, None),
            scroll_dist: 0.0,
            mouse_pressed_down_on_index: None,
            height_of_element: RelOrAbs::Rel(0.05),
            layout_content_data: EditorWindowLayoutContentData::default(),
        }
    }
}
impl EditorWindowLayoutContentTrait for VideoPropertiesEditor {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {

        for action in input.get_custom_actions().unwrap() {
            match action {
                CustomDrawActions::VideoPreviewResize(_) => (),
                CustomDrawActions::SetEditingTo(new_index) => self.editing = (match new_index {
                    Some(new) => Some((
                        new.clone(),
                        useful::get_elem_from_index_recursive_mut(&mut self.video.lock().unwrap(), &mut new.clone()).unwrap().clone_no_caching(),
                        VideoChanges::default()
                    )),
                    None => None,
                }, Some(Instant::now())),
            };
        };

        match &draw_opts.draw_mode.clone() /* TODO: Can you not clone here? */ {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    self.draw_normal(1.0, draw_opts, graphics, position, input);
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    draw_type_preview(1.0, if *moving { 1.0 } else { 0.5 }, graphics, position);
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => match modes {
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => {},
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                    draw_type_preview(*prog, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                    draw_type_preview(1.0 - prog, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old, }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new, }] => {
                    if *moving_old == *moving_new {
                        draw_type_preview(1.0, if *moving_new { 1.0 } else { 0.0 }, graphics, position)
                    } else if *moving_new {
                        draw_type_preview(1.0, *prog, graphics, position)
                    } else {
                        draw_type_preview(1.0, 1.0 - prog, graphics, position)
                    }
                },
            },
        };
        fn draw_type_preview(vis: f32, moving: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32)) {
                fn pt(x: f32, y: f32, d: &(f32, f32, f32, f32)) -> Vector2<f32> { Vector2::new(d.0 + x * d.2, d.1 + y * d.3) }
                let clr = Color::from_rgba(1.0 - moving * 0.3, 1.0 - moving * 0.3, 1.0, vis);
                graphics.draw_line(
                    pt(0.3, 0.2, &position),
                    pt(0.3, 0.8, &position),
                    2.5 - moving, clr);
                for i in 0..20 {
                    let y = 0.3 + 0.025 * i as f32;
                    graphics.draw_line(
                        pt(0.3, y, &position),
                        pt(0.3 + 0.025 * i as f32, y, &position),
                        1.0, clr);
                };
        }
    }

    fn handle_input_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None |
            crate::gui::speedy2d::layout::InputAction::Keyboard(_) => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                crate::gui::speedy2d::layout::MouseAction::Moved =>{
                    if let Some(_) = self.check_mouse_index_changed(draw_opts, input) {
                        self.mouse_pressed_down_on_index = None;
                    };
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => match btn {
                    speedy2d::window::MouseButton::Middle |
                    speedy2d::window::MouseButton::Right |
                    speedy2d::window::MouseButton::Other(_) => (),
                    speedy2d::window::MouseButton::Left => {
                        if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
                            self.set_mouse_index_if_changed(draw_opts, input);
                        } else {
                            self.mouse_pressed_down_on_index = None;
                        };
                    },
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonUp(btn) => match btn {
                    speedy2d::window::MouseButton::Middle |
                    speedy2d::window::MouseButton::Right |
                    speedy2d::window::MouseButton::Other(_) => (),
                    speedy2d::window::MouseButton::Left => {
                        if self.check_mouse_index_changed(draw_opts, input).is_none() {
                            // if the index was changed, the action is invalidated
                        };
                        self.mouse_pressed_down_on_index = None;
                    },
                },
                crate::gui::speedy2d::layout::MouseAction::Scroll(dist) => if let EditorWindowLayoutContentDrawMode::Static(EditorWindowLayoutContentSDrawMode::Normal) = draw_opts.draw_mode {
                    if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
                        match dist {
                            speedy2d::window::MouseScrollDistance::Lines { x: _, y, z: _ } => {
                                self.scroll_dist += *y as f32; // 1 line = 1 element
                            },
                            speedy2d::window::MouseScrollDistance::Pixels { x: _, y, z: _ } => {
                                // height_of_elements_abs pixels = 1 element
                                self.scroll_dist += *y as f32 / self.get_height_of_element_abs(draw_opts.my_size_in_pixels.1);
                            },
                            speedy2d::window::MouseScrollDistance::Pages { x: _, y, z: _ } => {
                                // 1 page = amount of elements visible
                                self.scroll_dist += *y as f32 / self.get_height_of_element_rel(draw_opts.my_size_in_pixels.1);
                            },
                        };
                    };
                },
            },
        };
    }
    
    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::VideoPropertiesEditor(self).into()
    }

    fn as_window_title(&self) -> String {
        format!("properties editor")
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent] {
        &mut []
    }
}
impl VideoPropertiesEditor {
    fn draw_normal(&mut self, vis: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let txt = draw_opts.assets_manager.get_default_font().layout_text(
            match &self.editing.0 {
                Some((_, vid, _)) => {
                    let mut s = "Editing: ".to_string(); s.push_str(
                    match &vid.video.vt {
                        VideoTypeEnum::List(_) => "List",
                        VideoTypeEnum::WithEffect(_, _) => "Effect",
                        VideoTypeEnum::Image(_) => "Image",
                        VideoTypeEnum::Raw(_) => "Video",
                    });
                    s
                },
                None => "Nothing to edit".to_string(),
            }.as_str()
            , self.get_height_of_element_abs(position.3) * 0.8, TextOptions::new()
        );
        let x_offset = (position.2 - txt.width()) / 2.0;
        graphics.draw_text(Vector2::new(position.0 + x_offset, position.1), Color::from_rgba(1.0, 1.0, 1.0, 1.0), &txt);
    }

    fn check_mouse_index_changed(&self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) -> Option<Option<u32>> {
        let mouse_in_range = 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0;
        match (self.mouse_pressed_down_on_index, mouse_in_range) {
            (None, false) => None,
            (Some(_), false) => Some(None),
            (old, true) => {
                let new = Some(((input.clonable.mouse_pos.1 / self.get_height_of_element_rel(draw_opts.my_size_in_pixels.1)) - self.scroll_dist).max(0.0).floor() as u32);
                if old != new {
                    Some(new)
                } else {
                    None
                }
            },
        }
    }
    fn set_mouse_index_if_changed(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) -> bool {
        if let Some(new) = self.check_mouse_index_changed(draw_opts, input) {
            self.mouse_pressed_down_on_index = new;
            true
        } else { false }
    }

    fn get_height_of_element_rel(&self, height: f32) -> f32 {
        match &self.height_of_element {
            RelOrAbs::Rel(v) => v.clone(),
            RelOrAbs::Abs(v) => v / height,
        }
    }
    
    fn get_height_of_element_abs(&self, height: f32) -> f32 {
        match &self.height_of_element {
            RelOrAbs::Rel(v) => v * height,
            RelOrAbs::Abs(v) => v.clone(),
        }
    }
}