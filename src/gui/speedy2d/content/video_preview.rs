use speedy2d::{dimen::Vector2, color::Color, image::{ImageDataType, ImageSmoothingMode}, shape::Rectangle};

use crate::{multithreading::automatically_cache_frames::VideoWithAutoCache, gui::speedy2d::{layout::{CustomDrawActions, InputAction, EditorWindowLayoutContentData}, content_list::EditorWindowLayoutContentEnum}};

use super::super::layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode};

use speedy2d::font::TextLayout;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

pub struct VideoPreview {
    time_created: std::time::Instant,
    video: VideoWithAutoCache,
    video_position: (f32, f32, f32, f32),
    progress: f64,
    mouse_pos: Option<(f32, f32)>,
    mouse_on_progress_bar: Option<f64>,
    mouse_left_button_down_started_on_progress_bar: bool,
    draw_extra_info: Option<Option<(f32, f32, f32, f32, Vec<std::rc::Rc<speedy2d::font::FormattedTextBlock>>)>>,
    layout_content_data: EditorWindowLayoutContentData,
    size: (u32, u32, Option<(f32, f32, std::time::Instant)>),
}
impl VideoPreview {
    pub fn new(vid: std::sync::Arc<std::sync::Mutex<crate::video::Video>>) -> Self {
        Self {
            time_created: std::time::Instant::now(),
            video: VideoWithAutoCache::start(vid),
            video_position: (0.0, 0.0, 1.0, 0.95),
            progress: 0.0, mouse_pos: None, mouse_on_progress_bar: None, mouse_left_button_down_started_on_progress_bar: false,
            draw_extra_info: None,
            layout_content_data: EditorWindowLayoutContentData::default(),
            size: (0, 0, None),
        }
    }
}
impl VideoPreview {
    fn get_pos_in_video(&self, pos: (f32, f32)) -> (f32, f32) {
        (
            (pos.0 - self.video_position.0) / self.video_position.2,
            (pos.1 - self.video_position.1) / self.video_position.3
        )
    }
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
    fn draw_type_normal(&mut self, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), visibility: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions) {
        let resized = if let Some(size) = &mut self.size.2 {
            if size.0 != position.2 || size.1 != position.3 {
                size.0 = position.2;
                size.1 = position.3;
                size.2 = std::time::Instant::now();
                None
            } else if size.2.elapsed().as_secs_f64() > 0.25 {
                Some((size.0, size.1))
            } else {
                None
            }
        } else if self.size.0 != position.2 as u32 || self.size.1 != (position.3 * self.video_position.3) as u32 {
            self.size.2 = Some((position.2, position.3, std::time::Instant::now()));
            None
        } else {
            None
        };
        if let Some(size) = resized {
            let (new_width, new_height) = ((size.0 * self.video_position.2) as u32, (size.1 * self.video_position.3) as u32);
            if self.size.0 != new_width || self.size.1 != new_height {
                self.size = (new_width, new_height, None);
                self.video.set_width_and_height(new_width, new_height);
            } else {
                self.size.2 = None;
            }
        }

        {
            let img = &self.video.shared.lock().unwrap().frame;
            if let Some(img) = img {
                match graphics.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(img.width(), img.height()), img.as_bytes()) {
                    Ok(handle) => {
                        let x = position.0 + self.video_position.0 * position.2;
                        let y = position.1 + self.video_position.1 * position.3;
                        let w = self.video_position.2 * position.2;
                        let h = self.video_position.3 * position.3;
                        graphics.draw_rectangle_image_tinted(Rectangle::new(Vector2::new(x, y), Vector2::new(x + w, y + h)), Color::from_rgba(1.0, 1.0, 1.0, visibility), &handle);
                            // 0.95
                    },
                    Err(e) => println!("COULDN'T DRAW IMAGE TO PREVIEW: create_image_from_raw_pixels failed (speedy2d/content/video_preview.rs: draw_type_normal()): {e}"),
                }
            }
        }

        let progress_bar_width = 0.8 * {
            let invisibility = 1.0 - visibility;
            1.0 - invisibility * invisibility * invisibility
        };

        let progress_bar_space_on_side = (1.0 - progress_bar_width) / 2.0 * position.2;
        let progress_bar_width = progress_bar_width * position.2;
        let progress_bar_line_y = position.1 + position.3 * 0.97;
        let progress_bar_pos_x = position.0 + progress_bar_space_on_side + progress_bar_width * self.progress as f32;
        graphics.draw_line(Vector2::new(position.0 + progress_bar_space_on_side, progress_bar_line_y), Vector2::new(progress_bar_pos_x, progress_bar_line_y), visibility, Color::CYAN);
        graphics.draw_line(Vector2::new(progress_bar_pos_x, progress_bar_line_y), Vector2::new(position.0 + progress_bar_space_on_side + progress_bar_width, progress_bar_line_y), visibility, Color::BLUE);
        // extra info
        'draw_extra_info: {
            if let Some(mut pot_pos) = std::mem::replace(&mut self.draw_extra_info, None) {
                let pos = match (&pot_pos, &self.mouse_pos) {
                    (Some(pos), _) => (pos.0, pos.1),
                    (None, Some(pos)) => (pos.0 * position.2, pos.1 * position.3),
                    (None, None) => (0.0, 0.0),
                };
                let mouse = Vector2 { x: position.0 + pos.0, y: position.1 + pos.1 };
                let margins = 5.0;
                let pos_in_video = self.get_pos_in_video((pos.0 / position.2, pos.1 / position.3));
                let font = draw_opts.assets_manager.get_default_font();
                let txt = font.layout_text(
                    format!(
                        "Time: {}\nPos: {:.3} | {:.3}",
                        self.progress,
                        pos_in_video.0, pos_in_video.1
                    ).as_str(),
                    15.0, speedy2d::font::TextOptions::new());
                let (mut w, h) = (txt.width(), txt.height());
                let mut x1 = mouse.x.max(position.0);
                let mut y1 = (mouse.y - margins - h - margins).max(position.1);
                let mut y2 = y1 + margins + h + margins;
                // extra buttons and stuff
                if let Some(pot_pos) = &mut pot_pos {
                    if pot_pos.2 > w { w = pot_pos.2; };
                    y2 += pot_pos.3;
                };
                // ensure the info box doesn't leave my area
                let mut x2 = x1 + margins + w + margins;
                if x2 > position.0 + position.2 {
                    let diff = x2 - position.0 - position.2;
                    x2 -= diff;
                    x1 -= diff;
                    if x1 < position.0 { break 'draw_extra_info; }
                }
                if y2 > position.1 + position.3 {
                    let diff = y2 - position.1 - position.3;
                    y2 -= diff;
                    y1 -= diff;
                    if y1 < position.1 { break 'draw_extra_info; }
                }
                graphics.draw_rectangle(Rectangle::new(Vector2 { x: x1, y: y1 }, Vector2 { x: x2, y: y2 }), Color::from_rgba(0.2, 0.2, 0.3, 0.75));
                graphics.draw_text(Vector2 { x: x1 + margins, y: y1 + margins }, Color::from_rgb(1.0, 1.0, 1.0), &txt);
                if let Some((_x, y, _w, h, buttons)) = &pot_pos {
                    let mut y = *y;
                    let item_height = h / buttons.len().max(1) as f32;
                    for button in buttons.iter() {
                        graphics.draw_text(Vector2 { x: x1 + margins, y }, Color::from_rgb(1.0, 1.0, 1.0), button);
                        y += item_height;
                    }
                }
                self.draw_extra_info = Some(pot_pos);
            }
        }
    }
}
impl EditorWindowLayoutContentTrait for VideoPreview {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {

        {
            for action in input.get_custom_actions().unwrap().iter() {
                match action {
                    CustomDrawActions::SetVideoPreviewActive(false) => { self.video.pause(true); },
                    CustomDrawActions::SetVideoPreviewActive(true) => { self.video.resume(); },
                    CustomDrawActions::SetEditingTo(_) => (), // TODO?
                    CustomDrawActions::ChangedVideo => {
                        self.video.pause(true); // clear cache
                        self.video.resume(); // start producing frames again
                    },
                };
            };
        };

        if draw_opts.force_redraw_due_to_resize {
            //self.update_with_resolution(self.size.0, self.size.1);
        };
        match &draw_opts.draw_mode.clone() {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    self.draw_type_normal(graphics, position, draw_opts.visibility, draw_opts);
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, draw_opts.visibility, graphics, position)
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => {
                match modes {
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => {
                        self.draw_type_normal(graphics, position, draw_opts.visibility, draw_opts);
                    },
                    [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                        self.draw_type_normal(graphics, position, (1.0 - prog) * draw_opts.visibility, draw_opts);
                        self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, prog * draw_opts.visibility, graphics, position);
                    },
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                        self.draw_type_normal(graphics, position, prog * draw_opts.visibility, draw_opts);
                        self.draw_type_preview(if *moving { 1.0 } else { 0.0 }, (1.0 - prog) * draw_opts.visibility, graphics, position);
                    },
                    [EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_old, }, EditorWindowLayoutContentSDrawMode::TypePreview { moving: moving_new, }] => {
                        if *moving_old == *moving_new {
                            self.draw_type_preview(if *moving_new { 1.0 } else { 0.0 }, draw_opts.visibility, graphics, position)
                        } else if *moving_new {
                            self.draw_type_preview(*prog, draw_opts.visibility, graphics, position)
                        } else {
                            self.draw_type_preview(1.0 - prog, draw_opts.visibility, graphics, position)
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
                    self.mouse_pos = Some(mouse_pos);
                    fn on_progress_bar(mouse_pos: (f32, f32)) -> bool {
                        (mouse_pos.1 - 0.97).abs() <= 0.02 && mouse_pos.0 >= 0.05 && mouse_pos.0 <= 0.95
                    }
                    let mouse_held_from_progress_bar = self.mouse_left_button_down_started_on_progress_bar && input.owned.mouse_down_buttons.contains_key(&speedy2d::window::MouseButton::Left);
                    self.mouse_on_progress_bar = if mouse_held_from_progress_bar || on_progress_bar(mouse_pos) {
                        let prog = ((mouse_pos.0 as f64 - 0.1) / 0.8).max(0.0).min(1.0);
                        if mouse_held_from_progress_bar { self.progress = prog; self.video.set_desired_progress(prog); };
                        Some(prog)
                    } else { None };
                },
                crate::gui::speedy2d::layout::MouseAction::ButtonDown(btn) => {
                    self.mouse_left_button_down_started_on_progress_bar = false;
                    if let Some(Some((popup_x, popup_y, popup_w, popup_h, actions))) = &self.draw_extra_info {
                        let (mouse_x, mouse_y) = (input.clonable.mouse_pos.0 * draw_opts.my_size_in_pixels.0, input.clonable.mouse_pos.1 * draw_opts.my_size_in_pixels.1);
                        if mouse_x >= *popup_x && mouse_x <= *popup_x + *popup_w {
                            let which = (mouse_y - popup_y) / popup_h;
                            if which >= 0.0 && which <= 1.0 {
                                let which = (which * actions.len() as f32) as usize;
                                match which {
                                    0 => {
                                        let cb: Result<clipboard::ClipboardContext, _> = clipboard::ClipboardProvider::new();
                                        match cb {
                                            Ok(mut cb) => if let Err(e) = cb.set_contents(format!("{}", self.progress)) { println!("Cannot set clipboard contents: {e}"); },
                                            Err(e) => println!("Cannot access clipboard: {e}"),
                                        }
                                    },
                                    1 => {
                                        let cb: Result<clipboard::ClipboardContext, _> = clipboard::ClipboardProvider::new();
                                        match cb {
                                            Ok(mut cb) => {
                                                let pos = self.get_pos_in_video((popup_x / draw_opts.my_size_in_pixels.0, popup_y / draw_opts.my_size_in_pixels.1));
                                                if let Err(e) = cb.set_contents(format!("{}/{}", pos.0, pos.1)) { println!("Cannot set clipboard contents: {e}"); };
                                            },
                                            Err(e) => println!("Cannot access clipboard: {e}"),
                                        }
                                    },
                                    _ => {},
                                }
                            }
                        }
                        self.draw_extra_info = None;
                    } else {
                        let pos = self.get_pos_in_video(self.mouse_pos.unwrap_or((-1.0, -1.0)));
                        if pos.0 >= 0.0 && pos.0 <= 1.0 && pos.1 >= 0.0 && pos.1 < 1.0 {
                            if match btn { speedy2d::window::MouseButton::Right => true, _ => false } {
                                let font = draw_opts.assets_manager.get_default_font();
                                let item_height = font.layout_text("|", 15.0, speedy2d::font::TextOptions::new()).height();
                                let buttons: Vec<_> = ["copy time", "copy position"].into_iter().map(|i| font.layout_text(i, 15.0, speedy2d::font::TextOptions::new())).collect();
                                let mut width = 0.0;
                                for btn in buttons.iter() {
                                    let btnw = btn.width();
                                    if btnw > width { width = btnw; }
                                }
                                let height = item_height * buttons.len() as f32;
                                let mp = if let Some(mp) = &self.mouse_pos { (mp.0 * draw_opts.my_size_in_pixels.0, mp.1 * draw_opts.my_size_in_pixels.1) } else { (0.0, 0.0) };
                                self.draw_extra_info = Some(Some((mp.0, mp.1, width, height, buttons)));
                            }
                        }
                    }
                    match self.mouse_on_progress_bar {
                        Some(prog) => match btn {
                            speedy2d::window::MouseButton::Left => {self.progress = prog; self.video.set_desired_progress(prog); self.mouse_left_button_down_started_on_progress_bar = true; }
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
            InputAction::Keyboard(action) => match action {
                crate::gui::speedy2d::layout::KeyboardAction::Pressed(kc, _) => {
                    // println!("Pressed {kc}");
                    // draw extra info
                    if input.owned.keyboard_modifiers_state.shift() && self.draw_extra_info.is_none() {
                        self.draw_extra_info = Some(None);
                    }
                    match (input.owned.keyboard_modifiers_state.logo(), input.owned.keyboard_modifiers_state.ctrl(), input.owned.keyboard_modifiers_state.alt(), input.owned.keyboard_modifiers_state.shift(), kc) {
                        _ => (),
                    }
                },
                crate::gui::speedy2d::layout::KeyboardAction::Released(_, _) => {
                    // draw extra info
                    if let Some(s) = &self.draw_extra_info {
                        if s.is_none() && ! input.owned.keyboard_modifiers_state.shift() {
                            self.draw_extra_info = None;
                        }
                    }
                },
                crate::gui::speedy2d::layout::KeyboardAction::Typed(_ch) => {},
            },
        }
    }
    
    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::VideoPreview(self).into()
    }
    fn as_enum_type(&self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::VideoPreview
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
