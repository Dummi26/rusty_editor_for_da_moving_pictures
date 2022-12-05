use core::num;
use std::{sync::{Arc, Mutex}, time::{Instant, Duration}, path::{PathBuf, Path}};

use egui::TextBuffer;
use speedy2d::{dimen::Vector2, color::Color, font::{TextLayout, TextOptions, TextAlignment}, shape::Rectangle};

use crate::{video::{Video, VideoChanges, VideoTypeEnum, VideoTypeChanges, VideoTypeChanges_List, VideoType}, content::{content::Content, image::ImageChanges}, gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentData, CustomDrawActions, MouseAction}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}, request::EditorWindowLayoutRequest}, effect::{self, effects::{EffectsEnum, EffectT}}, useful, curve::Curve};

type StringTypeForExtraTabTitle<'a> = std::borrow::Cow<'a, str>;

pub struct VideoPropertiesEditor {
    video: Arc<Mutex<Video>>,
    editing: (Option<(u32, Video)>, Option<Instant>),
    scroll_dist: f32,
    prev_scroll_dist: f32,
    height_of_element: RelOrAbs,

    tab_hover_change_duration: Duration,
    tab_change_duration: Duration,

    has_keyboard_focus: bool,

    tab: (usize, Option<(usize, Instant)>),
    tabs: Vec<Option<Box<dyn ExtraTabsInfo>>>,
    tabs_info: Vec<AnyTabInfo>,

    layout_content_data: EditorWindowLayoutContentData,
}
#[derive(Default)]
struct AnyTabInfo {
    hovered: bool,
    hovered_changed: Option<Instant>,
}

// pub enum ExtraTabsInfoEnum {
//     General,
//     StartAndLength(f64, f64),
//     ChangeType,
//     Curve { name: String, id: u32, write_changes: fn(&mut Video, Curve), curve: Curve, },
// 
//     ListEdit,
//     ListAdd,
// 
//     ImagePath(PathBuf, bool),
// }

struct EtGeneral {
} impl EtGeneral {
    pub fn new() -> Self {
        Self {
        }
    }
} impl ExtraTabsInfo for EtGeneral {
    fn title(&self) -> StringTypeForExtraTabTitle { "general".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0 + (hovered + selected).min(1.0) * position.2, y: position.1 }, Vector2 { x: position.0 + position.2 - selected * position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        let opts = ["x-position", "y-position", "width", "height", "time", "change type"];
        let options = opts.len();
        let h = position.3 / options as f32;
        for (opt, txt) in opts.into_iter().enumerate() {
            let y = position.1 + position.3 * opt as f32 / options as f32;
            graphics.draw_line(Vector2 { x: position.0, y, }, Vector2 { x: position.0 + position.2, y, }, 1.0, Color::from_rgba(1.0, 1.0, 1.0, vis));
            graphics.draw_text(Vector2 { x: position.0, y: y + 0.25 * h, }, Color::from_rgba(1.0, 1.0, 1.0, vis), &draw_opts.assets_manager.get_default_font().layout_text(txt, h * 0.5, TextOptions::new()));
        };
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None |
            crate::gui::speedy2d::layout::InputAction::Keyboard(_) => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                MouseAction::Moved => (),
                MouseAction::ButtonDown(_) => (),
                MouseAction::ButtonUp(btn) => {
                    let possibilities = 6;
                    let mouse_pos = input.clonable.mouse_pos;
                    let mouse_index = if 0.0 < mouse_pos.0 && mouse_pos.0 < 1.0 && 0.0 < mouse_pos.1 && mouse_pos.1 < 1.0 {
                        Some(((mouse_pos.1 * possibilities as f32).floor() as usize).min(possibilities - 1))
                    } else { None };
                    if let Some(mouse_index) = mouse_index {
                        match mouse_index {
                            0 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    supr.tabs.push(Some(Box::new(EtCurve::new("x".to_string(),
                                        0, |v, c| { v
                                        .set_pos.x = c; }, supr.editing.0.as_ref().unwrap().1
                                        .set_pos.x.clone()))));
                                };
                            },
                            1 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    supr.tabs.push(Some(Box::new(EtCurve::new("y".to_string(),
                                        1, |v, c| { v
                                        .set_pos.y = c; }, supr.editing.0.as_ref().unwrap().1
                                        .set_pos.y.clone()))));
                                };
                            },
                            2 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    supr.tabs.push(Some(Box::new(EtCurve::new("width".to_string(),
                                        2, |v, c| { v
                                        .set_pos.w = c; }, supr.editing.0.as_ref().unwrap().1
                                        .set_pos.w.clone()))));
                                };
                            },
                            3 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    supr.tabs.push(Some(Box::new(EtCurve::new("height".to_string(),
                                        3, |v, c| { v
                                        .set_pos.h = c; }, supr.editing.0.as_ref().unwrap().1
                                        .set_pos.h.clone()))));
                                };
                            },
                            4 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    let editing = supr.editing.0.as_ref().unwrap();
                                    supr.tabs.push(Some(Box::new(EtTime::new_len(editing.1.set_start_frame, editing.1.set_length))));
                                };
                            },
                            5 => {
                                if let Some(index) = VideoPropertiesEditor::get_extra_tabs_index_where(&supr.tabs, |e| false) {
                                    supr.change_tab_to(index, false);
                                } else {
                                    supr.change_tab_to(supr.tabs.len(), false);
                                    supr.tabs.push(Some(Box::new(EtChangeType::new())));
                                };
                            },
                            _ => (),
                        };
                    };
                },
                MouseAction::Scroll(_) => (),
            },
        };
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(_) => (),
        }
    }
}

struct EtTime {
    pub start: f64,
    pub end: f64,
} impl EtTime {
    pub fn new_len(start: f64, length: f64) -> Self {
        Self {
            start,
            end: start + length,
        }
    }
} impl ExtraTabsInfo for EtTime {
    fn title(&self) -> StringTypeForExtraTabTitle { "edit start and end time".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0, y: position.1 }, Vector2 { x: position.0 + position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        let per_item_height = supr.get_height_of_element_abs(draw_opts.my_size_in_pixels.1); // TODO figure out what this is, why it is, and then decide if something needs to be changed
        let font = draw_opts.assets_manager.get_default_font();
        let y = position.1 + position.3 * 0.375;
        let editing_video = &supr.editing.0.as_ref().unwrap().1;
        graphics.draw_line(Vector2 { x: position.0 + 0.05 * position.2, y, }, Vector2 { x: position.0 + (0.05 + 0.9 * self.start as f32) * position.2, y, }, per_item_height * 0.1, Color::from_rgba(if editing_video.set_start_frame == self.start { 0.5 } else { 1.0 }, 0.5, 0.5, vis));
        let y = position.1 + position.3 * 0.625;
        graphics.draw_line(Vector2 { x: position.0 + 0.05 * position.2, y, }, Vector2 { x: position.0 + (0.05 + 0.9 * self.end as f32) * position.2, y, }, per_item_height * 0.1, Color::from_rgba(if editing_video.set_length == self.end - self.start { 0.5 } else { 1.0 }, 0.5, 0.5, vis));
        let text = font.layout_text(format!("from {}\nto {}", self.start, self.end).as_str(), per_item_height * 0.9, TextOptions::new().with_wrap_to_width(position.2, TextAlignment::Center));
        graphics.draw_text(Vector2 { x: position.0, y: position.1, }, Color::from_rgba(1.0, 1.0, 1.0, vis), &text);
        let text = font.layout_text("click to apply", per_item_height * 0.9, TextOptions::new().with_wrap_to_width(position.2, TextAlignment::Center));
        graphics.draw_text(Vector2 { x: position.0, y: position.1 + position.3 - text.height(), }, Color::from_rgba(1.0, 1.0, 1.0, vis), &text);
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        let mouse_pos = input.clonable.mouse_pos;
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                MouseAction::Moved | MouseAction::ButtonUp(speedy2d::window::MouseButton::Left) => {
                    if input.owned.mouse_down_buttons.contains_key(&speedy2d::window::MouseButton::Left) {
                        if 0.0 < mouse_pos.0 && mouse_pos.0 < 1.0 && 0.0 < mouse_pos.1 && mouse_pos.1 < 1.0 {
                            match (mouse_pos.1 * 4.0).floor() as i32 {
                                0 => {},
                                1 => self.start = ((mouse_pos.0 as f64 - 0.05) / 0.9).max(0.0).min(self.end),
                                2 => self.end = ((mouse_pos.0 as f64 - 0.05) / 0.9).max(self.start).min(1.0),
                                3 => {
                                    supr.data().requests.push(EditorWindowLayoutRequest::EditingChangesApply(
                                        VideoChanges {
                                            pos: None, start: Some(self.start), length: Some(self.end - self.start),
                                            video: None, wrap: None, replace: None,
                                        }
                                    ));
                                },
                                _ => (),
                            };
                        };
                    };
                },
                _ => (),
            }
            _ => (),
        }
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(vid) => {
                self.start = vid.set_start_frame;
                self.end = self.start + vid.set_length;
            },
        }
    }
}

struct EtChangeType {
    possibilities: Option<Vec<(String, Option<crate::video::VideoChangesReplaceWith>)>>,
} impl EtChangeType {
    pub fn new() -> Self {
        Self {
            possibilities: None,
        }
    }
} impl ExtraTabsInfo for EtChangeType {
    fn title(&self) -> StringTypeForExtraTabTitle { "change type".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0, y: position.1 }, Vector2 { x: position.0 + position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        if self.possibilities.is_none() {
            if let Some((_, editing)) = &supr.editing.0 {
                self.possibilities = Some(match &editing.video.vt {
                    VideoTypeEnum::List(..) => vec![
                        ("first element in aspect ratio".to_string(), Some(crate::video::VideoChangesReplaceWith::AspectRatio)),
                        ("first element with effect".to_string(), Some(crate::video::VideoChangesReplaceWith::WithEffect)),
                    ],
                    VideoTypeEnum::AspectRatio(..) => vec![
                        ("list".to_string(), Some(crate::video::VideoChangesReplaceWith::List)),
                    ],
                    VideoTypeEnum::WithEffect(..) => vec![
                        ("list".to_string(), Some(crate::video::VideoChangesReplaceWith::List)),
                    ],
                    VideoTypeEnum::Text(t) => match t.text() {
                        crate::content::text::TextType::Static(_) => vec![
                            ("image (using this as its path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Image)),
                            ("video (with this as its path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Ffmpeg)),
                        ],
                    },
                    VideoTypeEnum::Image(..) => vec![
                        ("text (showing the path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Text)),
                        ("video (with the same path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Ffmpeg)),
                    ],
                    VideoTypeEnum::Raw(..) => vec![
                        ("text (showing the path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Text)),
                        ("video (with the same path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Ffmpeg)),
                        ("image (with the same path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Image)),
                    ],
                    VideoTypeEnum::Ffmpeg(..) => vec![
                        ("text (showing the path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Text)),
                        ("image (with the same path)".to_string(), Some(crate::video::VideoChangesReplaceWith::Image)),
                    ],
                });
            }
        }
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        if let Some(possibilities) = &self.possibilities {
            if ! possibilities.is_empty() {
                let element_height = position.3 / possibilities.len() as f32;
                for (i, possibility) in possibilities.iter().enumerate() {
                    let y = position.1 + position.3 * i as f32 / possibilities.len() as f32;
                    let txt = draw_opts.assets_manager.get_default_font().layout_text(possibility.0.as_str(), 0.5 * element_height, TextOptions::new());
                    graphics.draw_text(Vector2 { x: position.0, y: y + element_height * 0.25 }, Color::from_rgba(1.0, 1.0, 1.0, vis), &txt);
                }
            }
        }
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                MouseAction::Moved => (),
                MouseAction::ButtonDown(btn) => match btn {
                    _ => (),
                },
                MouseAction::ButtonUp(btn) => match btn {
                    speedy2d::window::MouseButton::Left => {
                        if let Some(possibilities) = &self.possibilities {
                            if possibilities.len() > 0 {
                                let (mx, my) = input.clonable.mouse_pos.clone();
                                println!("Mouse @ {}|{}", mx, my);
                                if mx >= 0.0 && mx <= 1.0 && my >= 0.0 && my <= 1.0 {
                                    if let Some((_, Some(p))) = possibilities.get((my * possibilities.len() as f32) as usize) {
                                        let changes = VideoChanges { pos: None, start: None, length: None, video: None, wrap: None, replace: Some(p.clone()), };
                                        supr.data().requests.push(EditorWindowLayoutRequest::EditingChangesApply(changes));
                                    }
                                }
                            }
                        }
                    },
                    _ => (),
                },
                MouseAction::Scroll(_) => (),
            },
            crate::gui::speedy2d::layout::InputAction::Keyboard(action) => match action {
                _ => (),
            },
        }
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(_) => self.possibilities = None,
        }
    }
}

struct EtCurve {
    name: String,
    id: u32,
    write_changes: fn(&mut Video, Curve),
    curve: Curve,
} impl EtCurve {
    pub fn new(name: String, id: u32, write_changes: fn(&mut Video, Curve), curve: Curve) -> Self {
        Self {
            name,
            id,
            write_changes,
            curve,
        }
    }
} impl ExtraTabsInfo for EtCurve {
    fn title(&self) -> StringTypeForExtraTabTitle { format!("edit {}-curve", self.name).into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0, y: position.1 }, Vector2 { x: position.0 + position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        let diagram_width = position.2.ceil() as _;
        let diagram_width_minus_one_float = (diagram_width - 1) as f64;
        let mut values = Vec::with_capacity(diagram_width);
        let (mut min, mut max) = (0.0, 1.0);
        for i in 0..diagram_width {
            let v = self.curve.get_value(i as f64 / diagram_width_minus_one_float);
            if v > max { max = v; }
            if v < min { min = v; }
            values.push(v);
        }
        let minmaxdiff = max - min;
        let mut prev_point = None;
        { // draw 0.0 and 1.0 lines
            let left = position.0;
            let right = position.0 + position.2;
            let y = position.1;
            let h = position.3;
            let y1 = y + h * ((max - 1.0) / minmaxdiff) as f32;
            let y2 = y + h * (max / minmaxdiff) as f32;
            graphics.draw_line(Vector2 { x: left, y: y1 }, Vector2 { x: right, y: y1 },
            1.0, Color::from_rgba(0.7, 1.0, 0.7, vis));
            graphics.draw_line(Vector2 { x: left, y: y2 }, Vector2 { x: right, y: y2 },
            1.0, Color::from_rgba(0.7, 0.7, 1.0, vis));
        }
        for (i, v) in values.iter().enumerate() {
            let v = if vis < 1.0 { // on fadein/fadeout, smoothly fade to a flat line of y=0.5
                0.5 + (v - 0.5) * (vis * vis) as f64
            } else { *v };
            let mut pos_y = position.1 + position.3 * ((max - v) / minmaxdiff) as f32;
            let this_vec = Vector2 { x: position.0 + i as f32, y: pos_y };
            if let Some(prev) = prev_point.take() {
                graphics.draw_line(
                    prev,
                    this_vec,
                    1.0,
                    Color::from_rgba(1.0, 1.0, 1.0, vis),
                );
            }
            prev_point = Some(this_vec);
            
        }
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(vid) => () /* TODO */,
        }
    }
}

struct EtListAdd {
} impl EtListAdd {
    pub fn new() -> Self {
        Self {
        }
    }
} impl ExtraTabsInfo for EtListAdd {
    fn title(&self) -> StringTypeForExtraTabTitle { "add a video to the list".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        // DRAW: LIST: ADD
        let possibilities = [
            ("List", "Holds multiple objects, applying its own size and position to all of them."),
            ("Effect", "Applies effects to an object."),
            ("Image", "Displays a static image"),
            ("Video", "Displays a video."),
        ];
        let mouse_index = if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
            Some(((input.clonable.mouse_pos.1 * possibilities.len() as f32).floor() as usize).min(possibilities.len() - 1))
        } else { None };
        let h = position.3 / possibilities.len() as f32;
        for (possibility, text) in possibilities.into_iter().enumerate() {
            let hover = Some(possibility) == mouse_index;
            let y = position.1 + position.3 * possibility as f32 / possibilities.len() as f32;
            graphics.draw_line(Vector2 { x: position.0, y, }, Vector2 { x: position.0 + position.2, y: y, }, 1.0, Color::from_rgba(0.5, 0.5, 0.5, vis));
            graphics.draw_text(Vector2 { x: position.0, y, }, Color::from_rgba(if hover { 1.0 } else { 0.7 }, if hover { 1.0 } else { 0.7 }, if hover { 1.0 } else { 0.7 }, vis), &draw_opts.assets_manager.get_default_font().layout_text(text.0, h * 0.5, TextOptions::new()));
            graphics.draw_text(Vector2 { x: position.0, y: y + 0.5 * h, }, Color::from_rgba(0.7, 0.7, 0.7, vis), &draw_opts.assets_manager.get_default_font().layout_text(text.1, h * 0.25, TextOptions::new().with_wrap_to_width(position.2, TextAlignment::Left)));
        };
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        let mouse_pos = input.clonable.mouse_pos;
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None |
            crate::gui::speedy2d::layout::InputAction::Keyboard(_) => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                MouseAction::Moved => (),
                MouseAction::ButtonDown(_) => (),
                MouseAction::ButtonUp(btn) => {
                    let possibilities = 4;
                    let mouse_index = if 0.0 < mouse_pos.0 && mouse_pos.0 < 1.0 && 0.0 < mouse_pos.1 && mouse_pos.1 < 1.0 {
                        Some(((mouse_pos.1 * possibilities as f32).floor() as usize).min(possibilities - 1))
                    } else { None };
                    if let Some(mouse_index) = mouse_index {
                        supr.change_tab_to(1, false);
                        let inner_changes = match mouse_index {
                            0 => Some(VideoTypeChanges_List::Insert(0, Video::new_full(VideoType::new(VideoTypeEnum::List(Vec::new()))))),
                            1 => Some(VideoTypeChanges_List::Insert(0, Video::new_full(VideoType::new(VideoTypeEnum::WithEffect(Box::new(Video::new_full(VideoType::new(VideoTypeEnum::List(Vec::new())))), effect::Effect::new(effect::effects::Nothing::new().as_enum())))))),
                            2 => Some(VideoTypeChanges_List::Insert(0, Video::new_full(VideoType::new(VideoTypeEnum::Image(crate::content::image::Image::new(PathBuf::from("/"))))))),
                            3 => Some(VideoTypeChanges_List::Insert(0, Video::new_full(VideoType::new(VideoTypeEnum::Raw(crate::content::input_video::InputVideo::new()))))),
                            _ => None,
                        };
                        if let Some(inner_changes) = inner_changes {
                            let changes = VideoChanges { pos: None, start: None, length: None, video: Some(VideoTypeChanges::List(vec![inner_changes])), wrap: None, replace: None, };
                            supr.data().requests.push(EditorWindowLayoutRequest::EditingChangesApply(changes));
                        };
                    };
                },
                MouseAction::Scroll(_) => (),
            },
        };
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(_) => (),
        }
    }
}

struct EtPath {
    og_path: PathBuf,
    pub path: PathBuf,
    ends_in_path_sep: bool,
} impl EtPath {
    pub fn new_vid(vid: &Video) -> Self {
        let path = Self::get_path(vid);
        Self { og_path: path.clone(), path, ends_in_path_sep: false, }
    }
    pub fn new_path(path: PathBuf) -> Self {
        Self {
            og_path: path.clone(),
            path,
            ends_in_path_sep: false,
        }
    }
    fn get_path(vid: &Video) -> PathBuf {
        match &vid.video.vt {
            VideoTypeEnum::Image(img) => img.path().clone(),
            VideoTypeEnum::Raw(vid) => vid.get_dir().clone(),
            _ => PathBuf::from("!! invalid video type !!"),
        }
    }
} impl ExtraTabsInfo for EtPath {
    fn title(&self) -> StringTypeForExtraTabTitle { "edit path".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0, y: position.1 }, Vector2 { x: position.0 + position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let font = draw_opts.assets_manager.get_default_font();
        let vis = draw_opts.visibility_factors.video_properties_editor_tabs;
        let per_item_height = supr.get_height_of_element_abs(draw_opts.my_size_in_pixels.1); // TODO: what is this?
        let path_text = font.layout_text(&self.path.to_string_lossy(), per_item_height * 0.7, TextOptions::new().with_wrap_to_width(position.2, TextAlignment::Left));
        let color = match (supr.has_keyboard_focus, self.path != self.og_path) {
            (false, false) => Color::from_rgba(
                0.8,
                0.8,
                0.8,
                vis
            ),
            (true, false) => Color::from_rgba(
                1.0,
                1.0,
                1.0,
                vis
            ),
            (true, true) => Color::from_rgba(
                1.0,
                0.8,
                0.8,
                vis
            ),
            (false, true) => Color::from_rgba(
                1.0,
                0.5,
                0.5,
                vis
            ),
        };
        graphics.draw_text(Vector2 { x: position.0, y: position.1 }, color, &path_text);
    }
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(_) => (),
            crate::gui::speedy2d::layout::InputAction::Keyboard(action) => match action {
                crate::gui::speedy2d::layout::KeyboardAction::Pressed(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Released(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Typed(ch) => match useful::CharOrAction::from(ch) {
                    useful::CharOrAction::Char(ch) => match ch {
                        '/' | '\\' => {
                            self.path.push("");
                            self.ends_in_path_sep = true;
                        },
                        _ => if self.ends_in_path_sep {
                            self.path.push(ch.to_string());
                            self.ends_in_path_sep = false;
                        } else {
                            let mut name = match self.path.file_name() {
                                Some(s) => s.to_string_lossy().to_string(),
                                None => String::new(),
                            };
                            name.push(ch);
                            self.path.set_file_name(name);
                        },
                    },
                    useful::CharOrAction::Enter => {
                        supr.data().requests.push(EditorWindowLayoutRequest::EditingChangesApply(VideoChanges {
                            video: Some(VideoTypeChanges::Image(ImageChanges {
                                path: Some(self.path.clone()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }));
                        // self.og_path = self.path.clone(); // TODO: necessary? or just wait for next frame where it should update anyway (would prevent false information in weird fail-/edgecases)
                    },
                    useful::CharOrAction::Backspace => {
                        let mut name = match self.path.file_name() { Some(s) => s.to_string_lossy().to_string(), None => String::new() };
                        if !self.ends_in_path_sep { name.pop(); };
                        if name.len() == 0 {
                            self.ends_in_path_sep = true;
                        };
                        self.path.set_file_name(name);
                    },
                    useful::CharOrAction::Delete => (),
                    useful::CharOrAction::Tab => (),
                    useful::CharOrAction::Ignored => (),
                },
            },
        };
    }
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {
        match ud {
            ETUpdate::VideoWasUpdated(vid) => self.og_path = Self::get_path(vid),
        }
    }
}

struct EtPlaceholder {
} impl EtPlaceholder {
    pub fn new() -> Self {
        Self {
        }
    }
} impl ExtraTabsInfo for EtPlaceholder {
    fn title(&self) -> StringTypeForExtraTabTitle { "PLACEHOLDER".into() }
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData) {
        graphics.draw_line(Vector2 { x: position.0, y: position.1 }, Vector2 { x: position.0 + position.2, y: position.1 + position.3 }, 1.0, shared_data.unified_color);
    }
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {}
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {}
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate) {}
}

trait ExtraTabsInfo {
    fn title(&self) -> StringTypeForExtraTabTitle;
    fn draw_icon(&mut self, vis: f32, hovered: f32, selected: f32, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), shared_data: &SharedEtData);
    fn draw(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput);
    fn handle_input(&mut self, supr: &mut VideoPropertiesEditor, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput);
    fn update(&mut self, supr: &mut VideoPropertiesEditor, ud: ETUpdate);
}
enum ETUpdate<'a> {
    VideoWasUpdated(&'a Video),
}
struct SharedEtData {
    pub unified_color: Color,
}

//

//

//

//

//

//

//

//

//

//

enum RelOrAbs { Rel(f32), Abs(f32), }

impl VideoPropertiesEditor {
    pub fn new(video: Arc<Mutex<Video>>) -> Self {
        Self {
            video,
            editing: (None, None),
            scroll_dist: 0.0,
            prev_scroll_dist: 0.0,
            height_of_element: RelOrAbs::Abs(24.0),

            tab_hover_change_duration: Duration::from_secs_f64(0.2),
            tab_change_duration: Duration::from_secs_f64(0.3),

            has_keyboard_focus: false,

            tab: (0, None),
            tabs: Vec::new(),
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
                CustomDrawActions::SetVideoPreviewActive(_) => (),
                val @ CustomDrawActions::SetEditingTo(_) |
                val @ CustomDrawActions::ChangedVideo => {
                    let new_index = if let CustomDrawActions::SetEditingTo(new_index) = val { *new_index } else { if let Some(ed) = &self.editing.0 { Some(ed.0) } else { None } };
                    self.editing = (
                        if let Some(index) = new_index {
                            let is_new_vid = if let Some(editing) = &self.editing.0 {
                                editing.0 != index
                            } else {
                                true
                            };
                            let new_vid = useful::get_elem_from_index_recursive_mut(&mut self.video.lock().unwrap(), &mut index.clone()).unwrap().clone_no_caching();
                            if is_new_vid {
                                self.tabs = match &new_vid.video.vt {
                                    VideoTypeEnum::List(_) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPlaceholder::new()))/*ExtraTabsInfo::ListEdit*/, Some(Box::new(EtListAdd::new()))/*ExtraTabsInfo::ListAdd*/],
                                    VideoTypeEnum::AspectRatio(..) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPlaceholder::new()))/*width*/, Some(Box::new(EtPlaceholder::new()))/*height*/],
                                    VideoTypeEnum::WithEffect(_, _) => vec![Some(Box::new(EtGeneral::new()))],
                                    VideoTypeEnum::Text(_) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPlaceholder::new()))],
                                    VideoTypeEnum::Image(img) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPath::new_path(img.path().clone())))/*ExtraTabsInfo::ImagePath(img.path().clone(), false)*/],
                                    VideoTypeEnum::Raw(_) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPlaceholder::new()))],
                                    VideoTypeEnum::Ffmpeg(_) => vec![Some(Box::new(EtGeneral::new())), Some(Box::new(EtPlaceholder::new()))],
                                };
                            } else {
                                let mut tabs = std::mem::replace(&mut self.tabs, Vec::new());
                                for tab in tabs.iter_mut() {
                                    if let Some(tab) = tab {
                                        tab.update(self, ETUpdate::VideoWasUpdated(&new_vid));
                                    }
                                }
                                self.tabs = tabs;
                            }
                            Some((index.clone(), new_vid))
                        } else {
                            self.change_tab_to(0, true);
                            None
                        },
                        Some(Instant::now())
                    );
                },
            };
        };

        match &draw_opts.draw_mode.clone() /* TODO: Can you not clone here? */ {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    self.draw_normal(draw_opts.visibility, draw_opts, graphics, position, input);
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    draw_type_preview(draw_opts.visibility, if *moving { 1.0 } else { 0.5 }, graphics, position);
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => match modes {
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => {},
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                    self.draw_normal((1.0 - prog) * draw_opts.visibility, draw_opts, graphics, position, input);
                    draw_type_preview(prog * draw_opts.visibility, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                    self.draw_normal(prog * draw_opts.visibility, draw_opts, graphics, position, input);
                    draw_type_preview((1.0 - prog) * draw_opts.visibility, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old, }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new, }] => {
                    if *moving_old == *moving_new {
                        draw_type_preview(draw_opts.visibility, if *moving_new { 1.0 } else { 0.0 }, graphics, position)
                    } else if *moving_new {
                        draw_type_preview(draw_opts.visibility, *prog, graphics, position)
                    } else {
                        draw_type_preview(draw_opts.visibility, 1.0 - prog, graphics, position)
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
        // global-er input
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None => (),
            crate::gui::speedy2d::layout::InputAction::Keyboard(_) => (),
            crate::gui::speedy2d::layout::InputAction::Mouse(action) => match action {
                crate::gui::speedy2d::layout::MouseAction::Moved => {
                    if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
                    } else {
                        self.has_keyboard_focus = false;
                    };
                    self.from_mouse_pos_adjust_highlighting_and_get_index(draw_opts, input);
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => match btn {
                    speedy2d::window::MouseButton::Middle |
                    speedy2d::window::MouseButton::Right |
                    speedy2d::window::MouseButton::Other(_) => (),
                    speedy2d::window::MouseButton::Left => {
                        if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && 0.0 < input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 < 1.0 {
                            self.has_keyboard_focus = true;
                        };
                        if let Some(index) = self.from_mouse_pos_adjust_highlighting_and_get_index(draw_opts, input) {
                            self.change_tab_to(index, false);
                        };
                    },
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonUp(btn) => match btn {
                    speedy2d::window::MouseButton::Middle |
                    speedy2d::window::MouseButton::Right |
                    speedy2d::window::MouseButton::Other(_) => (),
                    speedy2d::window::MouseButton::Left => {},
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

        // very local input
        let index = self.tab.0; // because we pass &mut self to tab.handle_input, self.tab.0 might change, which would break the vector array (leaving a None value somewhere and overwriting another innocent tab) - so we store the index in a var
        if let Some(tab) = self.tabs.get_mut(index) {
            if let Some(mut tab) = tab.take() {
                let mouse_pos = self.get_inner_mouse_position(draw_opts.my_size_in_pixels.1, &input.clonable.mouse_pos);
                let prev_mouse_pos = std::mem::replace(&mut input.clonable.mouse_pos, mouse_pos);
                tab.handle_input(self, draw_opts, input);
                self.tabs[index] = Some(tab);
                input.clonable.mouse_pos = prev_mouse_pos;
            }
        }
        // TODO: put this in trait objects
        // match self.editing.0.take() {
        //     Some(mut editing) => {
        //         let mut tabs = std::mem::replace(&mut self.tabs, Vec::new());
        //         let mouse_pos = self.get_inner_mouse_position(draw_opts.my_size_in_pixels.1, &input.clonable.mouse_pos);
        //         match (&mut tabs[self.tab.0], &mut editing.1.video.vt) {
        //             (ExtraTabsInfo::General, _) => {
        //             },
        //             (ExtraTabsInfo::StartAndLength(start, end), _) => {
        //             }
        //             (ExtraTabsInfo::ListAdd, VideoTypeEnum::List(_)) => {
        //             },
        //             (_, VideoTypeEnum::List(_)) => (),
        //             (_, VideoTypeEnum::WithEffect(_, _)) => (),
        //             (ExtraTabsInfo::ImagePath(path, ends_in_path_sep), VideoTypeEnum::Image(img)) => {
        //             },
        //             (ExtraTabsInfo::Curve { .. }, _) => {
        //                 /* TODO */
        //             },
        //             (_, VideoTypeEnum::Image(_)) => (),
        //             (_, VideoTypeEnum::Raw(_)) => (),
        //         };
        //         self.editing.0 = Some(editing);
        //         self.tabs = tabs;
        //     },
        //     None => (),
        // };
    }
    
    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::VideoPropertiesEditor(self).into()
    }
    fn as_enum_type(&self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::VideoPropertiesEditor
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

        let top_bar_height_rel = self.get_height_of_element_rel(position.3);
        let top_bar_height = self.get_height_of_element_abs(position.3);
        let tab_bar_height_rel = 2.0 * top_bar_height_rel;
        let tab_bar_height = 2.0 * top_bar_height;

        let position_inner = self.get_inner_position(position);

        if top_bar_height_rel + tab_bar_height_rel < 1.0 {
            let prev_mouse_y = input.clonable.mouse_pos.1;
            input.clonable.mouse_pos.1 = self.get_inner_mouse_position(draw_opts.my_size_in_pixels.1, &input.clonable.mouse_pos).1;
            match self.tab.clone() /* TODO: optimally, we wouldn't clone here. */ {
                (tab, Some((old_tab, time))) => {
                    let prog = time.elapsed().as_secs_f32() / self.tab_change_duration.as_secs_f32();
                    self.draw_tab(vis * prog, tab, self.scroll_dist, draw_opts, graphics, &position_inner, input);
                    self.draw_tab(vis * (1.0 - prog), old_tab, self.prev_scroll_dist, draw_opts, graphics, &position_inner, input);
                },
                (tab, None) => {
                    self.draw_tab(vis, tab, self.scroll_dist, draw_opts, graphics, &position_inner, input);
                },
            };
            input.clonable.mouse_pos.1 = prev_mouse_y;
        };

        let font = draw_opts.assets_manager.get_default_font();

        // draw the tab bar
        if !self.tabs.is_empty() {

            if self.tabs.len() <= self.tab.0 {
                self.change_tab_to(usize::MAX, false);
            };

            let w = position.2;
            let h = tab_bar_height;
            let x = position.0;
            let y_tab_bar_top = position.1 + position.3 - h;
            let num_of_tabs = self.tabs.len();
            if self.tabs_info.len() != num_of_tabs { // ensure self.tabs_info.len() == num_of_tabs
                while self.tabs_info.len() < num_of_tabs { self.tabs_info.push(AnyTabInfo::default()); };
                while self.tabs_info.len() > num_of_tabs { self.tabs_info.pop(); };
            };
            let (lines, tabs_per_line, line_height) = Self::get_tab_info(num_of_tabs, w, h);
            let mut index = 0;
            for line in 0..lines {
                let tabs_this_line = tabs_per_line.min(num_of_tabs - index);
                let y = y_tab_bar_top + line as f32 * line_height;
                graphics.draw_line(Vector2::new(x, y), Vector2::new(x + w, y), 1.0, Color::from_rgba(0.8, 0.8, 0.8, vis));
                let iw = line_height * tabs_this_line as f32;
                let x = match line % 3 {
                    1 => x + w - iw,
                    2 => x,
                    _ => x + (w - iw) / 2.0,
                };
                graphics.draw_line(Vector2 { x, y, }, Vector2 { x, y: y + line_height, }, 1.0, Color::from_rgba(0.6, 0.6, 0.6, vis));
                for tab_in_line in 0..tabs_this_line {
                    if index >= num_of_tabs { break; };
                    let tab = &mut self.tabs[index].as_mut().unwrap();
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
                    // TODO: Make good icons or smth idk
                    // let text = font.layout_text(match tab.draw_icon(1.0, graphics) {
                    //     ExtraTabsInfo::General => "G",
                    //     ExtraTabsInfo::StartAndLength(..) => "t",
                    //     ExtraTabsInfo::ChangeType => "ct",
                    //     ExtraTabsInfo::Curve {..} => "~",
                    //     ExtraTabsInfo::ListEdit => "L",
                    //     ExtraTabsInfo::ListAdd => "+",
                    //     ExtraTabsInfo::ImagePath(_, _) => "./",
                    // }, line_height * 0.75, TextOptions::new());
                    let (selected, reset_tab_old) = match &self.tab {
                        (tab, None) => (if index == *tab { 1.0 } else { 0.0 }, false),
                        (tab, Some((old_tab, time))) => {
                            if (index == *tab) != (index == *old_tab) {
                                let progress = (time.elapsed().as_secs_f32() / self.tab_change_duration.as_secs_f32()).min(1.0);
                                (if index == *tab {
                                    progress
                                } else {
                                    1.0 - progress
                                }, progress == 1.0)
                            } else { (0.0, false) }
                        },
                    };
                    if reset_tab_old { self.tab.1 = None; };
                    let tab_color = {
                        let selected = selected * 0.5;
                        Color::from_rgba(1.0 - selected - hovered * 0.5, 1.0 - selected, 1.0 - hovered * 0.5, vis)
                    };
                    if hovered > 0.0 {
                        let text = font.layout_text(tab.title().as_ref(), 0.45 * h, TextOptions::new());
                        graphics.draw_text(Vector2 { x: position.0 + (position.2 - text.width()) / 2.0, y: y_tab_bar_top - text.height(), }, Color::from_rgba(tab_color.r(), tab_color.g(), tab_color.b(), vis * hovered), &text)
                    }
                    // graphics.draw_text(Vector2 { x: x + (line_height - text.width()) / 2.0, y: y + 0.125 * line_height, }, tab_color, &text);
                    tab.draw_icon(vis, hovered, selected, graphics, &(x, y, line_height, line_height), &SharedEtData { unified_color: tab_color });
                    x += line_height;
                    graphics.draw_line(Vector2 { x: x, y: y, }, Vector2 { x: x, y: y + line_height, }, 1.0, Color::from_rgba(0.6, 0.6, 0.6, vis));
                    index += 1;
                };
            };
        };
        /* draw the header text */ {
            let txt = font.layout_text(
                match &self.editing.0 {
                    Some((_, vid)) => {
                        let mut s = "Editing: ".to_string(); s.push_str(
                        match &vid.video.vt {
                            VideoTypeEnum::List(_) => "List",
                            VideoTypeEnum::AspectRatio(..) => "AspectRatio",
                            VideoTypeEnum::WithEffect(_, _) => "Effect",
                            VideoTypeEnum::Text(_) => "Text",
                            VideoTypeEnum::Image(_) => "Image",
                            VideoTypeEnum::Raw(_) => "Video",
                            VideoTypeEnum::Ffmpeg(_) => "ffmpeg",
                        });
                        s
                    },
                    None => "Nothing to edit".to_string(),
                }.as_str()
                , top_bar_height * 0.8, TextOptions::new()
            );
            let x_offset = (position.2 - txt.width()) / 2.0;
            graphics.draw_text(Vector2::new(position.0 + x_offset, position.1), Color::from_rgba(1.0, 1.0, 1.0, vis), &txt);
        };
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

    fn draw_tab(&mut self, vis: f32, index: usize, scroll_dist: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        if let Some(tab) = self.tabs.get_mut(index) {
            if let Some(mut tab) = tab.take() {
                draw_opts.visibility_factors.video_properties_editor_tabs = vis;
                tab.draw(self, draw_opts, graphics, position, input);
                self.tabs[index] = Some(tab);
            }
        }
//         let per_item_height = self.get_height_of_element_abs(draw_opts.my_size_in_pixels.1);
//         let font = draw_opts.assets_manager.get_default_font();
//         match &mut self.editing.0 {
//             Some((editing_index, editing_video)) => {
// 
//                 if let Some(tab) = self.tabs.get(index) {
//                     match tab {
//                         ExtraTabsInfo::General => {
//                         },
//                         ExtraTabsInfo::StartAndLength(start, end) => {
//                         },
//                         ExtraTabsInfo::ListEdit => {
//                             if let VideoTypeEnum::List(vec) = &mut editing_video.video.vt {
//                                 // DRAW: LIST: EDIT
//                                 let per_item_height = 2.0 * per_item_height;
//                                 let mut y = position.1 - scroll_dist * per_item_height;
//                                 for child in vec {
//                                     if y >= -per_item_height {
//                                         let txt = font.layout_text(match &child.video.vt {
//                                             VideoTypeEnum::List(vec) => format!("List [{}]", vec.len()),
//                                             VideoTypeEnum::WithEffect(v, e) => format!("Effect"),
//                                             VideoTypeEnum::Image(_) => format!("Image"),
//                                             VideoTypeEnum::Raw(_) => format!("Video"),
//                                         }.as_str(), per_item_height * 0.7, TextOptions::new());
//                                         graphics.draw_text(Vector2 { x: position.0, y: y, }, Color::from_rgba(1.0, 1.0, 1.0, vis), &txt);
//                                     };
//                                     y += per_item_height;
//                                 };
//                             }
//                         },
//                         ExtraTabsInfo::ListAdd => {
//                         },
//                         ExtraTabsInfo::Curve { curve, .. } => {
//                         },
//                         ExtraTabsInfo::ImagePath(path, _) => {
//                         },
//                         _ => (),
//                     };
//                 };
//             },
//             None => (),
//         }
    }



    pub fn get_extra_tabs_index_where(tabs: &Vec<Option<Box<dyn ExtraTabsInfo>>>, f: fn(&dyn ExtraTabsInfo) -> bool) -> Option<usize> {
        for (i, extra_tab) in tabs.iter().enumerate() {
            if let Some(tab) = extra_tab {
                if f(tab.as_ref()) {
                    return Some(i);
                };
            }
        };
        return None;
    }
    pub fn get_tab_info(num_of_tabs: usize, width: f32, height: f32) -> (usize, usize, f32) {
        let space_for_tabs = width / height;
        let lines = (num_of_tabs as f32 / space_for_tabs).sqrt().ceil(); // if there are 4x as many tabs as there is space, we only need sqrt(4) = 2 lines because as we double the line count we also half the size and therefor the width of each tab.
        let line_height = height / lines;
        let tabs_per_line = (num_of_tabs as f32 / lines).ceil() as usize;
        let lines = lines as usize;

        (lines.into(), tabs_per_line, line_height)
    }
    
    pub fn get_inner_position(&self, position: &(f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
        let top_bar_height = self.get_height_of_element_abs(position.3);
        let tab_bar_height = 2.0 * top_bar_height;
        (position.0, position.1 + top_bar_height, position.2, position.3 - top_bar_height - tab_bar_height)
    }
    
    pub fn get_inner_mouse_position(&self, height: f32, mouse_pos: &(f32, f32)) -> (f32, f32) {
            let top_bar_height_rel = self.get_height_of_element_rel(height);
            let tab_bar_height_rel = 2.0 * top_bar_height_rel;
            (mouse_pos.0, (mouse_pos.1 - top_bar_height_rel) / (1.0 - top_bar_height_rel - tab_bar_height_rel))
    }

    /// Special case: if tab == usize::MAX, self.tabs.len()-1 will be used instead.
    pub fn change_tab_to(&mut self, mut tab: usize, instant: bool) {
        if tab == usize::MAX {
            tab = self.tabs.len() - 1;
        };
        if self.tab.0 != tab {
            self.tab = (tab, if instant { None } else { Some((self.tab.0, Instant::now())) });
            self.prev_scroll_dist = std::mem::replace(&mut self.scroll_dist, 0.0);
        };
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
                if i == 0 {
                }
                if i < self.tabs.len() {
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
