use std::{sync::{Arc, Mutex}, time::Instant};

use speedy2d::{dimen::Vector2, color::Color, font::{TextLayout, TextOptions}};

use crate::{video::Video, content::content::Content, gui::speedy2d::{layout::{EditorWindowLayoutContentTrait, EditorWindowLayoutContentDrawMode, EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentData, CustomDrawActions}, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}, request::EditorWindowLayoutRequest}, effect, useful};

pub struct VideoTree {
    /// This is always* the Some variant, so unwrapping it is safe.
    local_video_copy: Option<Video>,
    video: Arc<Mutex<Video>>,
    editing_index: (Option<u32>, Option<Instant>),
    scroll_dist: f32,
    mouse_pressed_down_on_index: Option<u32>,
    height_of_element: RelOrAbs,
    layout_content_data: EditorWindowLayoutContentData,
} enum RelOrAbs { Rel(f32), Abs(f32), }
impl VideoTree {
    pub fn new(video: Arc<Mutex<Video>>) -> Self {
        Self {
            local_video_copy: Some({ let v = video.lock().unwrap().clone_no_caching(); v.clone_no_caching() }),
            video,
            editing_index: (None, None),
            scroll_dist: 0.0,
            mouse_pressed_down_on_index: None,
            height_of_element: RelOrAbs::Abs(24.0),
            layout_content_data: EditorWindowLayoutContentData::default(),
        }
    }
}
impl EditorWindowLayoutContentTrait for VideoTree {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(&mut self, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {

        for action in input.get_custom_actions().unwrap() {
            match action {
                CustomDrawActions::SetVideoPreviewActive(_) => (),
                CustomDrawActions::SetEditingTo(new_index) => {
                    self.editing_index = (new_index.clone(), Some(Instant::now()));
                },
                CustomDrawActions::ChangedVideo => {
                    self.local_video_copy = Some(self.video.lock().unwrap().clone_no_caching());
                },
            };
        };

        match &draw_opts.draw_mode.clone() /* TODO: Can you not clone here? */ {
            EditorWindowLayoutContentDrawMode::Static(mode) => match mode {
                EditorWindowLayoutContentSDrawMode::Normal => {
                    self.draw_type_normal(draw_opts.visibility, draw_opts, graphics, position, input);
                },
                EditorWindowLayoutContentSDrawMode::TypePreview { moving } => {
                    draw_type_preview(draw_opts.visibility, if *moving { 1.0 } else { 0.5 }, graphics, position);
                },
            },
            EditorWindowLayoutContentDrawMode::Transition { modes, prog } => match modes {
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::Normal] => {},
                [EditorWindowLayoutContentSDrawMode::Normal, EditorWindowLayoutContentSDrawMode::TypePreview { moving }] => {
                    self.draw_type_normal((1.0 - prog) * draw_opts.visibility, draw_opts, graphics, position, input);
                    draw_type_preview(prog * draw_opts.visibility, if *moving { 1.0 } else { 0.0 }, graphics, position);
                },
                [EditorWindowLayoutContentSDrawMode::TypePreview { moving }, EditorWindowLayoutContentSDrawMode::Normal] => {
                    self.draw_type_normal(prog * draw_opts.visibility, draw_opts, graphics, position, input);
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
                for i in 0..5 {
                    let y = 0.3 + 0.1 * i as f32;
                    graphics.draw_line(
                        pt(0.3, y, &position),
                        pt(0.3 + 0.1 * i as f32, y, &position),
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
                            if let Some(index) = self.mouse_pressed_down_on_index { // this requires mouse to be in scope, so we do not need to check again.
                                if let Some(_) = useful::get_elem_from_index_recursive_mut(self.local_video_copy_mut(), &mut index.clone()) { // this requires the index to have been in range. if it wasn't, the mouse was clicked on some empty space.
                                    self.data().requests.push(EditorWindowLayoutRequest::SelectForEditing(index));
                                };
                            };
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
        EditorWindowLayoutContentEnum::VideoTree(self).into()
    }
    fn as_enum_type(&self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::VideoTree
    }

    fn as_window_title(&self) -> String {
        format!("tree view")
    }

    fn data(&mut self) -> &mut EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent] {
        &mut []
    }
}
impl VideoTree {
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

    fn draw_type_normal(&mut self, vis: f32, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), input: &mut crate::gui::speedy2d::layout::UserInput) {
        let sd = self.scroll_dist.clone();
        let mut vid = self.local_video_copy.take().unwrap();
        graphics.set_clip(Some(speedy2d::shape::Rectangle::from_tuples((position.0.ceil() as i32, position.1.ceil() as i32), ((position.0 + position.2).floor() as i32, (position.1 + position.3).floor() as i32))));
        self.draw_type_normal_one(vis, &mut vid, draw_opts, graphics, position, &mut (0.0, sd), &mut 0, input);
        graphics.set_clip(None);
        self.local_video_copy = Some(vid);
    }
    fn draw_type_normal_one(&mut self, vis: f32, vid: &mut Video, draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions, graphics: &mut speedy2d::Graphics2D, position: &(f32, f32, f32, f32), npos: &mut (f32, f32), index: &mut u32, input: &mut crate::gui::speedy2d::layout::UserInput) -> DrawTreeBranchRecursiveOptions {
        fn vid_to_str(vid: &Video) -> String {
            match &vid.video.vt {
                crate::video::VideoTypeEnum::List(_) => format!("List"),
                crate::video::VideoTypeEnum::AspectRatio(_, _w, _h) => format!("AspectRatio"),
                crate::video::VideoTypeEnum::WithEffect(_, e) => format!("Effect: {}", match &e.effect {
                    effect::effects::EffectsEnum::Nothing(_) => format!("Nothing"),
                    effect::effects::EffectsEnum::BlackWhite(_) => format!("BlackWhite"),
                    effect::effects::EffectsEnum::Rotate(_) => format!("Rotate: [?]"),
                    effect::effects::EffectsEnum::Shake(e) => format!("Shake ({}x{}, {}x{})", e.shake_dist_x, e.shakes_count_x, e.shake_dist_y, e.shakes_count_y),
                    effect::effects::EffectsEnum::ChangeTime(_) => format!("ChangeSpeed"),
                    effect::effects::EffectsEnum::ColorAdjust(e) => format!("ColorAdjust: {}", match &e.mode { effect::effects::ColorAdjust_Mode::Rgba(..) => "rgba", }),
                    effect::effects::EffectsEnum::Blur(e) => format!("Blur: {}", match &e.mode { effect::effects::Blur_Mode::Square {..} => "Square", effect::effects::Blur_Mode::Downscale {..} => "Downscale", }),
                    effect::effects::EffectsEnum::ColorKey(_) => format!("ColorKey"),
                }),
                crate::video::VideoTypeEnum::Text(t) => match t.text() {
                    crate::content::text::TextType::Static(txt) => format!("Text: \"{}\"", txt),
                    crate::content::text::TextType::Program(p) => format!("Text: from '{}'", p.path.to_string_lossy().as_ref()),
                },
                crate::video::VideoTypeEnum::Image(i) => format!("Image: {}", match i.path().file_name() { Some(n) => n.to_string_lossy().to_string(), None => i.path().to_string_lossy().to_string(), }),
                crate::video::VideoTypeEnum::Raw(i) => format!("Video: {}", i.get_dir().to_string_lossy().to_string()),
                crate::video::VideoTypeEnum::Ffmpeg(i) => format!("ffmpeg: {}", i.path().to_string_lossy().to_string()),
            }
        }

        let elem_height_rel = self.get_height_of_element_rel(draw_opts.my_size_in_pixels.1);
        let elem_height = self.get_height_of_element_abs(draw_opts.my_size_in_pixels.1);
        let elem_y_rel = elem_height_rel * npos.1;
        let elem_y = elem_height * npos.1;

        if elem_y_rel >= 1.0 { return DrawTreeBranchRecursiveOptions {
            did_draw: false,
            highlight_strength: 0.0,
        }; };

        let old_npos0 = npos.0.clone();
        let old_npos1 = npos.1.clone();
        npos.0 += 1.0;
        npos.1 += 1.0;

        let is_selected_for_editing = self.editing_index.0 == Some(*index);
        let is_almost_clicked_on = self.mouse_pressed_down_on_index == Some(*index);

        let mut recursive_options = {
            let mut highlight_strength = 0.0;
            for child in vid.children_mut() {
                *index += 1;
                let ropts = self.draw_type_normal_one(vis, child, draw_opts, graphics, position, npos, index, input);
                if ropts.highlight_strength > highlight_strength { highlight_strength = ropts.highlight_strength; };
            };
            DrawTreeBranchRecursiveOptions {
                did_draw: false,
                highlight_strength: highlight_strength / 2.0,
            }
        };

        if old_npos1 >= -1.0 {
            recursive_options.did_draw = true;
            if 0.0 < input.clonable.mouse_pos.0 && input.clonable.mouse_pos.0 < 1.0 && input.clonable.mouse_pos.1 > elem_y_rel && input.clonable.mouse_pos.1 <= elem_y_rel + elem_height_rel { // mouse hover
                recursive_options.highlight_strength = 1.0;
            }
            let pos_x = position.0 + position.2 * (0.025 * old_npos0 + 0.0125 * recursive_options.highlight_strength);
            let pos = Vector2::new(pos_x, position.1 + elem_y);
            let sel_for_edit_highlight = if is_selected_for_editing {
                Some(if let Some(sel_time) = self.editing_index.1 {
                    let cdown = 1.0 - sel_time.elapsed().as_secs_f32();
                    if cdown <= 0.0 {
                        self.editing_index.1 = None;
                        0.0
                    } else {
                        cdown * cdown * cdown * cdown * cdown * cdown * cdown
                    }
                } else { 0.0 })
            } else { None };
            let (r, g, b) = match (sel_for_edit_highlight, is_almost_clicked_on) {
                (None, false) => (1.0, 1.0, 1.0),
                (None, true) => (0.8, 0.8, 0.8),
                (Some(sel_highlight), false) => (0.3 + sel_highlight * 0.7, 0.3 + sel_highlight * 0.7, 1.0),
                (Some(sel_highlight), true) => (0.2 + sel_highlight * 0.8, 0.2 * sel_highlight * 0.8, 0.8),
            };
            graphics.draw_text(pos,
                Color::from_rgba(r, g, b, vis),
                &draw_opts.assets_manager.get_default_font().layout_text(vid_to_str(vid).as_str(), elem_height * 0.8, TextOptions::new())
            );
        };

        npos.0 = old_npos0;
        
        recursive_options
    }
    
    fn local_video_copy(&self) -> &Video {
        match &self.local_video_copy { Some(v) => v, None => panic!(), }
    }
    fn local_video_copy_mut(&mut self) -> &mut Video {
        match &mut self.local_video_copy { Some(v) => v, None => panic!(), }
    }
    
    fn get_elem_from_index(&self, mut index: u32) -> Option<&Video> {
        useful::get_elem_from_index_recursive(self.local_video_copy(), &mut index)
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
#[derive(Clone, Default)]
struct DrawTreeBranchRecursiveOptions {
    pub did_draw: bool,
    pub highlight_strength: f32,
}
