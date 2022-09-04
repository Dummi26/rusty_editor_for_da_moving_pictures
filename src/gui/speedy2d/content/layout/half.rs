use speedy2d::{dimen::Vector2, color::Color, window::{MouseButton, MouseScrollDistance}};

use crate::gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, InputAction, KeyboardAction, MouseAction, EditorWindowLayoutContentData, EditorWindowLayoutContentSDrawMode}, content::placeholder::Placeholder, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}};

pub struct Half {
    elems: [EditorWindowLayoutContent; 2],
    /// vertical = one above the other
    vertical: bool,
    split: f32,
    was_changed: bool,
    mouse_dragging_split_bar: bool,
    last_size_px: (f32, f32),
    layout_content_data: EditorWindowLayoutContentData,
}
impl Half { pub fn new_placeholders(vertical: bool, split: f32) -> Self { Self::new([Placeholder::new().as_enum(), Placeholder::new().as_enum()], vertical, split) } pub fn new(elems: [EditorWindowLayoutContent; 2], vertical: bool, split: f32) -> Self {
    Self { elems, vertical, split, was_changed: false, mouse_dragging_split_bar: false, last_size_px: (0.0, 0.0), layout_content_data: EditorWindowLayoutContentData::default(), }
} }
impl Half {
    pub fn set_split(&mut self, v: f32) {
        self.split = v.max(0.0).min(1.0);
        self.was_changed = true;
    }
}
impl EditorWindowLayoutContentTrait for Half {
    fn was_changed_custom(&self) -> bool {
        self.was_changed || self.elems[0].was_changed() || self.elems[1].was_changed()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        self.last_size_px = (position.2, position.3);
        // if split was changed, tell the two parts that their size might also have changed.
        if self.was_changed {
            self.was_changed = false;
            draw_opts.force_redraw_due_to_resize = true;
        };
        // modify the input
        let (first, second, is_first) = self.modify_mouse(input);
        let old = input.replace_clonable(first);
        // draw the parts
        let my_size_px = draw_opts.my_size_in_pixels.clone();
        if self.vertical {
            let top_bar_info = self.get_top_bar_infos(draw_opts);
            let split = self.get_split_with_top_bars(top_bar_info);
            // TODO: Get height of the two top bars, subtract first from top and second from bottom (so that if split = 1.0, there is still space for the top bar on the bottom, and for split = 0.0, the same applies for the top. This is necessary to prevent height from becoming negative after subtracting the top bar (which is automatically done))
            let h = split * position.3;
            draw_opts.my_size_in_pixels.1 = h;
            self.elems[0].draw_onto(draw_opts, graphics, &(
                position.0,
                position.1,
                position.2,
                h,
            ), input);
            input.replace_clonable(second);
            let h = position.3 - split * position.3;
            draw_opts.my_size_in_pixels.1 = h;
            self.elems[1].draw_onto(draw_opts, graphics, &(
                position.0,
                position.1 + split * position.3,
                position.2,
                h
                ), input);
        } else { // horizontal
            let w = self.split * position.2;
            draw_opts.my_size_in_pixels.0 = w;
            self.elems[0].draw_onto(draw_opts, graphics, &(
                position.0,
                position.1,
                w,
                position.3,
            ), input);
            input.replace_clonable(second);
            let w = position.2 - self.split * position.2;
            draw_opts.my_size_in_pixels.0 = w;
            self.elems[1].draw_onto(draw_opts, graphics, &(
                position.0 + self.split * position.2,
                position.1,
                w,
                position.3,
            ), input);
        };
        // revert clonable change
        draw_opts.my_size_in_pixels = my_size_px;
        input.replace_clonable(old);
        // draw the line that separates the two parts
        if self.vertical {
            let y = position.1 as f32 + self.get_split_with_top_bars(self.get_top_bar_infos(draw_opts)) * position.3 as f32;
            graphics.draw_line(
                Vector2::new(position.0 as f32, y),
                Vector2::new((position.0 + position.2) as f32, y),
                1.0, Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            );
        } else {
            let x = position.0 as f32 + self.split * position.2 as f32;
            graphics.draw_line(
                Vector2::new(x, position.1 as f32),
                Vector2::new(x , (position.1 + position.3) as f32),
                1.0, Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            );
        };
    }
    fn handle_input_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, input: &mut crate::gui::speedy2d::layout::UserInput) {
        {
            match draw_opts.draw_mode {
                EditorWindowLayoutContentDrawMode::Static(crate::gui::speedy2d::layout::EditorWindowLayoutContentSDrawMode::TypePreview { moving: false, }) => {
                    match &input.owned.action {
                        InputAction::None | InputAction::Keyboard(_) => (),
                        InputAction::Mouse(action) => match action {
                            MouseAction::ButtonDown(btn) => match btn {
                                MouseButton::Left => {
                                    if self.vertical {
                                        let top_bar_info = self.get_top_bar_infos(draw_opts);
                                        let mouse_split_val = input.clonable.mouse_pos.1;
                                        let mouse_split_diff = mouse_split_val - self.get_split_with_top_bars(top_bar_info);
                                        if mouse_split_diff.abs() < 0.01 {
                                            self.set_split_with_top_bars(mouse_split_val, top_bar_info);
                                            self.mouse_dragging_split_bar = true;
                                        };
                                    } else {
                                        let mouse_split_val = input.clonable.mouse_pos.0;
                                        let mouse_split_diff = mouse_split_val - self.split;
                                        if mouse_split_diff.abs() < 0.01 {
                                            self.set_split(mouse_split_val);
                                            self.mouse_dragging_split_bar = true;
                                        };
                                    };
                                },
                                _ => (),
                            },
                            MouseAction::ButtonUp(btn) => match btn {
                                MouseButton::Left => self.mouse_dragging_split_bar = false,
                                _ => (),
                            },
                            MouseAction::Moved => {
                                if self.mouse_dragging_split_bar {
                                    if self.vertical {
                                        self.set_split_with_top_bars(input.clonable.mouse_pos.1, self.get_top_bar_infos(draw_opts));
                                    } else {
                                        self.set_split(input.clonable.mouse_pos.0);
                                    };
                                };
                            },
                            MouseAction::Scroll(dist) => {
                                if 0.0 <= input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 <= 1.0 && 0.0 <= input.clonable.mouse_pos.1 && input.clonable.mouse_pos.1 <= 1.0 {
                                    let mouse_split_val = if self.vertical { input.clonable.mouse_pos.1 } else { input.clonable.mouse_pos.0 };
                                    match &self.elems[if mouse_split_val < self.split { 0 } else { 1 }].c {
                                        EditorWindowLayoutContentEnum::LayoutHalf(_) => (),
                                        _ => {
                                            self.set_split(self.split + match dist {
                                                MouseScrollDistance::Lines { x, y, z: _, } => {
                                                    0.01 * (x - y) as f32
                                                },
                                                MouseScrollDistance::Pixels { x, y, z: _, } => {
                                                    *x as f32 / self.last_size_px.0 - *y as f32 / self.last_size_px.1
                                                },
                                                MouseScrollDistance::Pages { x, y, z: _, } => {
                                                    (x - y) as f32
                                                },
                                            });
                                        },
                                    };
                                };
                            },
                        },
                    }
                },
                _ => (),
            };

            let (first, second, _) = self.modify_mouse(input);
            let psize_px = if self.vertical { draw_opts.my_size_in_pixels.1 } else { draw_opts.my_size_in_pixels.0 };
            *if self.vertical { &mut draw_opts.my_size_in_pixels.1 } else { &mut draw_opts.my_size_in_pixels.0 } = psize_px * self.split;
            let old = input.replace_clonable(first);
            self.elems[0].handle_input(draw_opts, input);

            *if self.vertical { &mut draw_opts.my_size_in_pixels.1 } else { &mut draw_opts.my_size_in_pixels.0 } = psize_px * (1.0 - self.split);
            input.replace_clonable(second);
            self.elems[1].handle_input(draw_opts, input);

            *if self.vertical { &mut draw_opts.my_size_in_pixels.1 } else { &mut draw_opts.my_size_in_pixels.0 } = psize_px;
            input.replace_clonable(old);
        };
    }
    
    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::LayoutHalf(Box::new(self)).into()
    }
    
    fn as_window_title(&self) -> String {
        format!("{} split", if self.vertical { "vertical" } else { "horizontal" })
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData { &mut self.layout_content_data }

    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent] {
        self.elems.as_mut_slice()
    }
}
impl Half {
    pub fn get_split(&self) -> f32 { self.split }
    fn get_top_bar_infos(&self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions) -> (f32, f32) {
        (
            self.elems[0].height_of_top_bar_in_type_preview_mode_respecting_draw_mode(draw_opts, false),
            self.elems[0].height_of_top_bar_in_type_preview_mode_respecting_draw_mode(draw_opts, false)
        )
    }
    fn get_split_with_top_bars(&self, top_bar_info: (f32, f32)) -> f32 {
        self.split * (1.0 - top_bar_info.0 - top_bar_info.1) + top_bar_info.1
    }
    fn set_split_with_top_bars(&mut self, inner_split: f32, top_bar_info: (f32, f32)) {
        self.set_split((inner_split - top_bar_info.1) / (1.0 - top_bar_info.0 - top_bar_info.1));
    }

    /// Returns (new_clonable_first, new_clonable_second, is_mouse_in_first).
    fn modify_mouse(&mut self, input: &mut crate::gui::speedy2d::layout::UserInput) -> (crate::gui::speedy2d::layout::UserInputClonable, crate::gui::speedy2d::layout::UserInputClonable, bool) {
        let mut clonable_first = input.clonable.clone();
        let mut clonable_second = input.clonable.clone();
        
        let is_first = if self.vertical {
            clonable_first.mouse_pos.1 /= self.split;
            clonable_second.mouse_pos.1 -= self.split;
            clonable_second.mouse_pos.1 /= 1.0 - self.split;
            input.clonable.mouse_pos.1 < self.split
        } else {
            clonable_first.mouse_pos.0 /= self.split;
            clonable_second.mouse_pos.0 -= self.split;
            clonable_second.mouse_pos.0 /= 1.0 - self.split;
            input.clonable.mouse_pos.0 < self.split
        };
        (clonable_first, clonable_second, is_first)
    }
}