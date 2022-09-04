use core::num;
use std::{sync::{Arc, Mutex}, time::{Instant, Duration}};

use speedy2d::{dimen::Vector2, color::Color, font::{TextLayout, TextOptions, TextAlignment}, shape::Rectangle};

use crate::{video::{Video, VideoChanges, VideoTypeEnum}, content::content::Content, gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentData, CustomDrawActions}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}, request::EditorWindowLayoutRequest}, effect, useful};

pub struct VideoPropertiesEditor {
    video: Arc<Mutex<Video>>,
    editing: (Option<(u32, Video, VideoChanges)>, Option<Instant>),
    scroll_dist: f32,
    mouse_pressed_down_on_index: Option<u32>,
    height_of_element: RelOrAbs,

    tab_hover_change_duration: Duration,

    tab: usize,
    tabs_info: Vec<AnyTabInfo>,

    layout_content_data: EditorWindowLayoutContentData,
}
#[derive(Default)]
struct AnyTabInfo {
    hovered: bool,
    hovered_changed: Option<Instant>,
}
enum RelOrAbs { Rel(f32), Abs(f32), }
impl VideoPropertiesEditor {
    pub fn new(video: Arc<Mutex<Video>>) -> Self {
        Self {
            video,
            editing: (None, None),
            scroll_dist: 0.0,
            mouse_pressed_down_on_index: None,
            height_of_element: RelOrAbs::Abs(24.0),

            tab_hover_change_duration: Duration::from_secs_f64(0.2),

            tab: 0,
            tabs_info: Vec::new(),

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
                    self.draw_normal(1.0 - prog, draw_opts, graphics, position, input);
                    draw_type_preview(*prog, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                    self.draw_normal(*prog, draw_opts, graphics, position, input);
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
                crate::gui::speedy2d::layout::MouseAction::Moved => {
                    self.from_mouse_pos_adjust_highlighting_and_get_index(draw_opts, input);
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => match btn {
                    speedy2d::window::MouseButton::Middle |
                    speedy2d::window::MouseButton::Right |
                    speedy2d::window::MouseButton::Other(_) => (),
                    speedy2d::window::MouseButton::Left => {
                        if let Some(index) = self.from_mouse_pos_adjust_highlighting_and_get_index(draw_opts, input) {
                            self.tab = index;
                        }
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

struct TabInfo {
    full_name: String,
    short_name: String,
}
impl TabInfo {
    pub fn new_general() -> Self {
        Self { full_name: "General".to_string(), short_name: "G".to_string(), }
    }
}

impl VideoPropertiesEditor {
    fn draw_normal(&mut self, vis: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {

        let font = draw_opts.assets_manager.get_default_font();

        let tabs = match &self.editing.0 {
            Some(editing) => {
                match &editing.1.video.vt {
                    VideoTypeEnum::List(_) => vec![
                        TabInfo::new_general(),
                        TabInfo { full_name: "List".to_string(), short_name: "L".to_string(), },
                    ],
                    VideoTypeEnum::WithEffect(_, _) => vec![
                        TabInfo::new_general(),
                        TabInfo { full_name: "Effect".to_string(), short_name: "E".to_string(), },
                        TabInfo { full_name: "Details".to_string(), short_name: "D".to_string(), },
                    ],
                    VideoTypeEnum::Image(_) => vec![
                        TabInfo::new_general(),
                        TabInfo { full_name: "Image".to_string(), short_name: "I".to_string(), },
                    ],
                    VideoTypeEnum::Raw(_) => vec![
                        TabInfo::new_general(),
                        TabInfo { full_name: "Video".to_string(), short_name: "V".to_string(), },
                    ],
                }
            },
            None => Vec::new(),
        };

        // draw the tab bar
        if !tabs.is_empty() {
            let w = position.2;
            let h = self.get_height_of_element_abs(position.3) * 2.0;
            let x = position.0;
            let y = position.1 + position.3 - h;
            let num_of_tabs = tabs.len();
            if self.tabs_info.len() != num_of_tabs { // ensure self.tabs_info.len() == num_of_tabs
                while self.tabs_info.len() < num_of_tabs { self.tabs_info.push(AnyTabInfo::default()); };
                while self.tabs_info.len() > num_of_tabs { self.tabs_info.pop(); };
            };
            let (lines, tabs_per_line, line_height) = Self::get_tab_info(num_of_tabs, w, h);
            let mut index = 0;
            for line in 0..lines {
                let tabs_this_line = tabs_per_line.min(num_of_tabs - index);
                let y = y + line as f32 * line_height;
                graphics.draw_line(Vector2::new(x, y), Vector2::new(x + w, y), 1.0, Color::from_rgba(0.8, 0.8, 0.8, vis));
                let iw = line_height * tabs_this_line as f32;
                let x = match line % 3 {
                    1 => x + w - iw,
                    2 => x,
                    _ => x + (w - iw) / 2.0,
                };
                graphics.draw_line(Vector2 { x: x, y: y, }, Vector2 { x: x, y: y + line_height, }, 1.0, Color::from_rgba(0.6, 0.6, 0.6, vis));
                for tab_in_line in 0..tabs_this_line {
                    if index >= num_of_tabs { break; };
                    let tab = &tabs[index];
                    let tab_info = &mut self.tabs_info[index];
                    let hovered = match (tab_info.hovered, tab_info.hovered_changed) {
                        (true, None) => 1.0,
                        (false, None) => 0.0,
                        (true, Some(t)) => {
                            let prog = t.elapsed().as_secs_f32() / self.tab_hover_change_duration.as_secs_f32();
                            if prog >= 1.0 {
                                tab_info.hovered_changed = None;
                                1.0
                            } else {
                                prog
                            }
                        },
                        (false, Some(t)) => {
                            let prog = t.elapsed().as_secs_f32() / self.tab_hover_change_duration.as_secs_f32();
                            if prog >= 1.0 {
                                tab_info.hovered_changed = None;
                                0.0
                            } else {
                                1.0 - prog
                            }
                        },
                    };
                    let mut x = x + tab_in_line as f32 * line_height;
                    let text = font.layout_text(tab.short_name.as_str(), line_height * 0.75, TextOptions::new());
                    let selected = if index == self.tab { 0.5 } else { 0.0 };
                    graphics.draw_text(Vector2 { x: x + (line_height - text.width()) / 2.0, y: y + 0.125 * line_height, }, Color::from_rgba(1.0 - selected - hovered * 0.5, 1.0 - selected, 1.0 - hovered * 0.5, vis), &text);
                    x += line_height;
                    graphics.draw_line(Vector2 { x: x, y: y, }, Vector2 { x: x, y: y + line_height, }, 1.0, Color::from_rgba(0.6, 0.6, 0.6, vis));
                    index += 1;
                };
            };
        };
        /* draw the header text */ {
            let txt = font.layout_text(
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
            graphics.draw_text(Vector2::new(position.0 + x_offset, position.1), Color::from_rgba(1.0, 1.0, 1.0, vis), &txt);
        };
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

    pub fn get_tab_info(num_of_tabs: usize, width: f32, height: f32) -> (usize, usize, f32) {
        let space_for_tabs = width / height;
        let lines = (num_of_tabs as f32 / space_for_tabs).sqrt().ceil(); // if there are 4x as many tabs as there is space, we only need sqrt(4) = 2 lines because as we double the line count we also half the size and therefor the width of each tab.
        let line_height = height / lines;
        let tabs_per_line = (num_of_tabs as f32 / lines).ceil() as usize;
        let lines = lines as usize;

        (lines.into(), tabs_per_line, line_height)
    }

    pub fn from_mouse_pos_adjust_highlighting_and_get_index(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) -> Option<usize> {
        let tab_row_height = self.get_height_of_element_rel(draw_opts.my_size_in_pixels.1) * 2.0;
        let y_top_border_tab_row = 1.0 - tab_row_height;
        if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && input.clonable.mouse_pos.1 > y_top_border_tab_row && input.clonable.mouse_pos.1 < 1.0 {

            let num_of_tabs = self.tabs_info.len();

            let tab_info = Self::get_tab_info(num_of_tabs, draw_opts.my_size_in_pixels.0, draw_opts.my_size_in_pixels.1 * tab_row_height);

            let tabs_per_row = tab_info.1;

            let tabs_row = ((input.clonable.mouse_pos.1 - y_top_border_tab_row) * tab_info.0 as f32 / tab_row_height).floor() as usize;

            let mouse_x_abs = input.clonable.mouse_pos.0 * draw_opts.my_size_in_pixels.0;

            let prev_rows_tabs = tabs_row * tabs_per_row;

            let tabs_in_this_row = tabs_per_row.min(num_of_tabs - prev_rows_tabs);
            let this_row_tabs_width = tabs_in_this_row as f32 * tab_info.2;

            let within_row_mouse_index = (match tabs_row % 3 {
                1 => {
                    mouse_x_abs - (draw_opts.my_size_in_pixels.0 - this_row_tabs_width)
                },
                2 => {
                    mouse_x_abs.clone()
                },
                _ => {
                    mouse_x_abs - (draw_opts.my_size_in_pixels.0 - this_row_tabs_width) / 2.0
                },
            } / tab_info.2).floor();

            let index = if within_row_mouse_index >= 0.0 { Some(prev_rows_tabs + within_row_mouse_index as usize) } else { None };

            let now = Instant::now();
            for (i, tab_info) in self.tabs_info.iter_mut().enumerate() {
                if Some(i) == index {
                    if !tab_info.hovered {
                        tab_info.hovered = true;
                        tab_info.hovered_changed = Some(now);
                    };
                } else if tab_info.hovered {
                    tab_info.hovered = false;
                    tab_info.hovered_changed = Some(now);
                };
            };

            if let Some(i) = index {
                if i < num_of_tabs {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }

        } else {
            let now = Instant::now();
            for tab_info in self.tabs_info.iter_mut() {
                if tab_info.hovered {
                    tab_info.hovered = false;
                    tab_info.hovered_changed = Some(now);
                };
            };
            None
        }
    }
}