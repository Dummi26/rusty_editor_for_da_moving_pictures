use std::{time::{Instant, Duration}, collections::HashMap, ops::Sub};

mod layout;
mod editor_window_settings;
mod request;
pub mod content;
pub mod content_list;

use crate::{cli::Clz, project::Project, files, gui::speedy2d::layout::{EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentTrait}, assets::AssetsManager, video::{VideoChanges, Video}};
use speedy2d::{window::{WindowHandler, VirtualKeyCode}, color::Color};

use self::{content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum}, layout::{EditorWindowLayoutContentDrawOptions, EditorWindowLayoutContentDrawMode, CustomDrawActions}, editor_window_settings::EditorWindowSettings};

pub fn main(args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    eprintln!("{}",
        Clz::starting("Launching speedy2d gui..."),
    );
    match speedy2d::Window::new_centered("rusty editor for da moving pictures", (1280, 720)) {
        Ok(window) => {
            let mut window_handler = EditorWindowHandler::new(&args);
            window_handler.start_caching_thread();
            window.run_loop(window_handler)
        },
        Err(err) => panic!("\n{}{}\n", Clz::error_info("Error creating window with speedy2d: "), Clz::error_details(err.to_string().as_str())),
    }
}



struct EditorWindowHandler {
    creation_time: Instant,
    start_time: Instant,
    size: (u32, u32),
    resized: bool,
    project: Project,
    
    settings: EditorWindowSettings,

    draw_opts: EditorWindowLayoutContentDrawOptions,
    draw_mode: EditorWindowHandlerDrawMode,
    
    dragged_window: Option<(EditorWindowLayoutContent, (f32, f32), (f32, f32))>,
    edited_part: Option<u32>,
    qvidrunner_state: (f32, bool, Option<Instant>, Duration),

    mouse_pos: (f32, f32),
    mouse_down_buttons: HashMap<speedy2d::window::MouseButton, MouseButtonDownInfo>,
    keyboard_modifiers_state: speedy2d::window::ModifiersState,
    keyboard_down_buttons_scancode: HashMap<u32, u8>,
    keyboard_down_buttons_virtualkeycode: HashMap<speedy2d::window::VirtualKeyCode, u8>,
    
    layout: EditorWindowLayoutContent,
    custom_actions: Vec<CustomDrawActions>,
}

//#[derive(Clone)]
pub enum EditorWindowHandlerDrawMode {
    Static(EditorWindowLayoutContentSDrawMode),
    Transition(EditorWindowLayoutContentSDrawMode, EditorWindowLayoutContentSDrawMode, Instant, Duration),
}
impl EditorWindowHandlerDrawMode {
    /// Changes self to be a Transition, with the previous self being used as the "old" mode. If self already is a transition, uses that transition's "new" mode as its "old" mode (cutting the other transition off in favor of the new one).
    pub fn start_transition(&mut self, new_mode: EditorWindowLayoutContentSDrawMode, duration: Duration) { self.start_transition_be(new_mode, duration, false) }
    pub fn start_transition_allow_reverse(&mut self, new_mode: EditorWindowLayoutContentSDrawMode, duration: Duration) { self.start_transition_be(new_mode, duration, true) }
    pub fn start_transition_be(&mut self, new_mode: EditorWindowLayoutContentSDrawMode, duration: Duration, allow_reverse: bool) {
        match self {
            Self::Static(mode) => {
                *self = Self::Transition(mode.clone(), new_mode, Instant::now(), duration);
            },
            Self::Transition(v_old, v_new, v_start, v_duration) => {
                let reverse = allow_reverse && *v_old == new_mode;
                let v_new = std::mem::replace(v_new, new_mode);
                *v_old = v_new;
                *v_start = if reverse {
                    let prev_progress = v_start.elapsed().as_secs_f64() / v_duration.as_secs_f64();
                    let new_progress = 1.0 - prev_progress.max(0.0).min(1.0);
                    Instant::now() - duration.mul_f64(new_progress)
                } else {
                    Instant::now()
                };
                *v_duration = duration;
            },
        }
    }
    /// Converter function. If self is Transition but the duration has elapsed, changes self to Static. If self is Transition, calculates progress from the given start_time and duration.
    pub fn get_draw_mode(&mut self) -> EditorWindowLayoutContentDrawMode {
        match self {
            EditorWindowHandlerDrawMode::Static(v) => EditorWindowLayoutContentDrawMode::Static(v.clone()),
            EditorWindowHandlerDrawMode::Transition(old, new, started, duration) => {
                let prog = started.elapsed().as_secs_f32() / duration.as_secs_f32();
                if prog > 1.0 {
                    let new_self = Self::Static(new.clone());
                    let new_new = new.clone();
                    *self = new_self;
                    EditorWindowLayoutContentDrawMode::Static(new_new)
                } else {
                    EditorWindowLayoutContentDrawMode::Transition { modes: [old.clone(), new.clone()], prog, }
                }
            },
        }
    }
}

impl EditorWindowHandler {
    pub fn new(args: &crate::cli::CustomArgs) -> Self {
        fn project_gen_new_empty() -> Project {
            Project { proj: crate::project::ProjectData { name: "Unnamed project".to_string(), path: None, render_settings_export: None, }, vid: crate::multithreading::automatically_cache_frames::VideoWithAutoCache::new(crate::video::Video::new_full(
                crate::video::VideoType::new(crate::video::VideoTypeEnum::List(Vec::new()))
            )), }
        }
        let mut project = match &args.project_path {
            Some(path) => {
                match files::file_handler::read_from_file(path) {
                    Ok(Ok(proj)) => {
                        eprintln!("{}",
                            Clz::progress("Loaded project."),
                        );
                        proj
                    },
                    Ok(Err(err)) => {
                        eprintln!("{}{}{}\n    {}\n{}",
                            Clz::error_info("Error parsing project file '"), Clz::error_cause(path.to_string_lossy().as_ref()), Clz::error_info("':"),
                            Clz::error_details(err.to_string().as_str()),
                            Clz::progress("Continuing with a new (empty) project."),
                        );
                        project_gen_new_empty()
                    },
                    Err(err) => {
                        eprintln!("{}{}{}\n    {}\n{}",
                            Clz::error_info("IO-Error loading project from file '"), Clz::error_cause(path.to_string_lossy().as_ref()), Clz::error_info("':"),
                            Clz::error_details(err.to_string().as_str()),
                            Clz::progress("Continuing with a new (empty) project."),
                        );
                        project_gen_new_empty()
                    }
                }
            },
            None => {
                eprintln!("{}",
                    Clz::progress("No path was provided, so a new (empty) project will be created."),
                );
                project_gen_new_empty()
            }
        };
        project.vid.cache();
        Self {
            creation_time: Instant::now(),
            start_time: Instant::now(),
            size: (0, 0),
            resized: false,

            settings: EditorWindowSettings::default(),

            draw_opts: EditorWindowLayoutContentDrawOptions {
                force_redraw_due_to_resize: false,
                render_canvas_size: (0, 0),
                my_size_in_pixels: (0.0, 0.0),
                assets_manager: AssetsManager::default(),
                draw_mode: EditorWindowLayoutContentDrawMode::Static(EditorWindowLayoutContentSDrawMode::Normal),
            },
            draw_mode: EditorWindowHandlerDrawMode::Static(EditorWindowLayoutContentSDrawMode::Normal),
            
            dragged_window: None,
            edited_part: None,
            qvidrunner_state: (0.1, false, None, Duration::from_secs_f64(0.25)),

            mouse_pos: (0.5, 0.5),
            mouse_down_buttons: HashMap::new(),
            keyboard_modifiers_state: speedy2d::window::ModifiersState::default(),
            keyboard_down_buttons_scancode: HashMap::new(),
            keyboard_down_buttons_virtualkeycode: HashMap::new(),
            custom_actions: Vec::new(),
            layout: content::layout::half::Half::new(
                [
                    content::layout::half::Half::new(
                        [
                            content::video_preview::VideoPreview::new(project.vid.clone()).as_enum(),
                            content::layout::half::Half::new([
                                content::video_tree::VideoTree::new(project.vid.get_vid_mutex_arc().clone()).as_enum(),
                                content::video_properties_editor::VideoPropertiesEditor::new(project.vid.get_vid_mutex_arc().clone()).as_enum()
                            ], true, 0.4
                            ).as_enum()
                        ], false, 0.8
                    ).as_enum(),
                    content::placeholder::Placeholder::new(
                    ).as_enum()
                ], true, 0.8
            ).as_enum(),
            project,
        }
    }
}

impl EditorWindowHandler {
    pub fn start_caching_thread(&mut self) {
        self.project.vid.cache();
    }
}

impl WindowHandler for EditorWindowHandler {
    fn on_start(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        info: speedy2d::window::WindowStartupInfo
    )
    {
        self.start_time = Instant::now();
        eprintln!("{}\n    {}{}\n    {}{}",
            Clz::completed("gui started."),
            Clz::completed_info("Scale factor:  "), Clz::completed_info(info.scale_factor().to_string().as_str()),
            Clz::completed_info("Viewport size: "), Clz::completed_info({ let s = info.viewport_size_pixels(); format!("{}x{}", s.x, s.y).as_str() }),
        );
    }

    fn on_user_event(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        user_event: ()
    )
    {
    }

    fn on_resize(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        size_pixels: speedy2d::dimen::Vector2<u32>
    )
    {
        fn megapixels<T>(w: T, h: T) -> f64 where T: Into<f64> {
            w.into() * h.into() / 1_000_000.0
        }
        eprintln!("{}{}",
            Clz::progress("Resized: "), Clz::progress(format!("{}x{} ({}MP)-> {}x{} ({}MP)", self.size.0, self.size.1, megapixels(self.size.0, self.size.1), size_pixels.x, size_pixels.y, megapixels(size_pixels.x, size_pixels.y)).as_str()),
        );
        self.size = (size_pixels.x, size_pixels.y);
        self.resized = true;
    }

    fn on_mouse_grab_status_changed(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        mouse_grabbed: bool
    )
    {
    }

    fn on_fullscreen_status_changed(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        fullscreen: bool
    )
    {
    }

    fn on_scale_factor_changed(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        scale_factor: f64
    )
    {
    }

    fn on_draw(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        graphics: &mut speedy2d::Graphics2D
    )
    {
        self.draw_opts.force_redraw_due_to_resize = self.resized;
        self.draw_opts.render_canvas_size = self.size.clone();
        self.draw_opts.my_size_in_pixels = (self.size.0 as f32, self.size.1 as f32);
        self.draw_opts.draw_mode = self.draw_mode.get_draw_mode();
        
        // Handle requests
        
        self.handle_requests();
        
        // Prepare for drawing (Mandatory)

        graphics.clear_screen(Color::BLACK);
        
        // Prepare for drawing (Additional)
        
        if let Some(anim_start_time) = self.qvidrunner_state.2 {
            if let EditorWindowLayoutContentEnum::LayoutHalf(half) = &mut self.layout.c {
                let progress = anim_start_time.elapsed().as_secs_f32() / self.qvidrunner_state.3.as_secs_f32();
                let (progress, last) = if progress >= 1.0 {
                    self.qvidrunner_state.2 = None;
                    (1.0, true)
                } else {
                    let p = 1.0 - progress;
                    (1.0 - (
                        p*p*p*p
                    ), false)
                };
                if self.qvidrunner_state.1 == true {
                    half.set_split(progress * self.qvidrunner_state.0);
                } else {
                    half.set_split((1.0 - progress) * self.qvidrunner_state.0);
                    if last {
                        self.layout = half.get_children()[1].take();
                    };
                }
            } else {
                
            }
        }
        
        
        // Draw (custom)

        let custom_actions = std::mem::replace(&mut self.custom_actions, Vec::new());
        let mut input = &mut layout::UserInput::new_with_actions(
            layout::UserInputOwned {
                action: layout::InputAction::None,
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &custom_actions,
            &mut self.custom_actions,
        );
        self.layout.draw_onto(&mut self.draw_opts, graphics, &(0.0, 0.0, self.size.0 as f32, self.size.1 as f32), &mut input);
        
        // Dragged window
        
        if let Some((dragged_window, size, grab_position)) = &mut self.dragged_window {
            let position = (
                self.size.0 as f32 * self.mouse_pos.0 - size.0 * grab_position.0,
                self.size.1 as f32 * self.mouse_pos.1 - size.1 * grab_position.1,
                size.0, size.1
            );
            dragged_window.draw_onto(&mut self.draw_opts, graphics, &position, &mut input);
        };
        
        // Reset stuff

        self.resized = false;
        helper.request_redraw();
    }

    fn on_mouse_move(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        position: speedy2d::dimen::Vector2<f32>
    )
    {
        self.mouse_pos = (position.x / self.size.0 as f32, position.y / self.size.1 as f32);
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Mouse(layout::MouseAction::Moved),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_mouse_button_down(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        button: speedy2d::window::MouseButton
    )
    {
        match self.mouse_down_buttons.get_mut(&button) {
            Some(v) => {},
            None => { self.mouse_down_buttons.insert(button, MouseButtonDownInfo {
                held_down_since: Instant::now(),
            }); },
        };
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Mouse(layout::MouseAction::ButtonDown(button)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_mouse_button_up(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        button: speedy2d::window::MouseButton
    )
    {
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Mouse(layout::MouseAction::ButtonUp(button)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
        self.mouse_down_buttons.remove(&button);
    }

    fn on_mouse_wheel_scroll(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        distance: speedy2d::window::MouseScrollDistance
    )
    {
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Mouse(layout::MouseAction::Scroll(distance)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_key_down(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
        scancode: speedy2d::window::KeyScancode
    )
    {
        match self.keyboard_down_buttons_scancode.get_mut(&scancode) {
            Some(v) => *v += 1,
            None => { self.keyboard_down_buttons_scancode.insert(scancode, 0); },
        };
        if let Some(vkc) = virtual_key_code {
            match self.keyboard_down_buttons_virtualkeycode.get_mut(&vkc) {
                Some(v) => *v += 1,
                None => { self.keyboard_down_buttons_virtualkeycode.insert(vkc, 0); },
            };
        };
        
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Keyboard(layout::KeyboardAction::Pressed(scancode, virtual_key_code)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_key_up(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
        scancode: speedy2d::window::KeyScancode
    )
    {
        match self.keyboard_down_buttons_scancode.get_mut(&scancode) {
            Some(v) => if *v == 0 {
                self.keyboard_down_buttons_scancode.remove(&scancode);
            } else { *v += 1 },
            None => (),
        };
        if let Some(vkc) = virtual_key_code {
            match self.keyboard_down_buttons_virtualkeycode.get_mut(&vkc) {
                Some(v) => if *v == 0 {
                self.keyboard_down_buttons_virtualkeycode.remove(&vkc);
            } else { *v += 1 },
                None => (),
            };
        };
        
        if let Some(vkc) = virtual_key_code {
            match vkc {
                VirtualKeyCode::Escape => {
                    match &self.draw_mode {
                        EditorWindowHandlerDrawMode::Static(mode) | EditorWindowHandlerDrawMode::Transition(_, mode, _, _) => match mode {
                            EditorWindowLayoutContentSDrawMode::Normal => self.draw_mode.start_transition_allow_reverse(EditorWindowLayoutContentSDrawMode::TypePreview { moving: false, }, self.settings.switch_modes_duration),
                            EditorWindowLayoutContentSDrawMode::TypePreview { moving: _ } => self.draw_mode.start_transition_allow_reverse(EditorWindowLayoutContentSDrawMode::Normal, self.settings.switch_modes_duration),
                        },
                    }
                },
                VirtualKeyCode::Space => {
                    if self.keyboard_modifiers_state.ctrl() {
                        if self.qvidrunner_state.1 == false {
                            self.qvidrunner_state.1 = true;
                            let layout = self.layout.take();
                            self.layout.replace(
                                content::layout::half::Half::new([
                                    content::special::qvidrunner::QVidRunner::new().as_enum(),
                                    layout,
                                ], true, 0.0).as_enum()
                            );
                        } else {
                            self.qvidrunner_state.1 = false;
                            if let EditorWindowLayoutContentEnum::LayoutHalf(half) = &mut self.layout.c {
                                self.qvidrunner_state.0 = half.get_split();
                            };
                        };
                        self.qvidrunner_state.2 = Some(Instant::now());
                    };
                },
                _ => (),
            };
        };
        
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Keyboard(layout::KeyboardAction::Released(scancode, virtual_key_code)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_keyboard_char(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        unicode_codepoint: char
    )
    {
        self.layout.handle_input(&mut self.draw_opts, &mut layout::UserInput::new_no_actions(
            layout::UserInputOwned {
                action: layout::InputAction::Keyboard(layout::KeyboardAction::Typed(unicode_codepoint)),
                project: &mut self.project,
                mouse_down_buttons: &self.mouse_down_buttons,
                keyboard_modifiers_state: &self.keyboard_modifiers_state,
                keyboard_down_buttons_scancode: &self.keyboard_down_buttons_scancode,
                keyboard_down_buttons_virtualkeycode: &self.keyboard_down_buttons_virtualkeycode,
            },
            layout::UserInputClonable {
                mouse_pos: self.mouse_pos.clone(),
            },
            &mut self.custom_actions,
        ));
    }

    fn on_keyboard_modifiers_changed(
        &mut self,
        helper: &mut speedy2d::window::WindowHelper<()>,
        state: speedy2d::window::ModifiersState
    )
    {
        self.keyboard_modifiers_state = state;
    }
    
    //
}

pub struct MouseButtonDownInfo {
    held_down_since: Instant,
}