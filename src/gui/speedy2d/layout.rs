use std::{collections::HashMap};

use speedy2d::{Graphics2D, window::{MouseButton, MouseScrollDistance, VirtualKeyCode, KeyScancode}};

use crate::{assets::AssetsManager};

use super::{content_list::{self, EditorWindowLayoutContent}, request::EditorWindowLayoutRequest};

pub trait EditorWindowLayoutContentTrait {
    /// If this returns true, draw_onto will be called the next frame. Your implementation of draw_onto should change your struct in such a way that was_changed returns false again (if appropriate).
    fn was_changed_custom(&self) -> bool;
    /// The draw function. To modify draw_opts, use clone().
    fn draw_onto_custom(&mut self, draw_opts: &mut EditorWindowLayoutContentDrawOptions, graphics: &mut Graphics2D, position: &(f32, f32, f32, f32), input: &mut UserInput);
    //
    fn handle_input_custom(&mut self, draw_opts: &mut EditorWindowLayoutContentDrawOptions, input: &mut UserInput);
    /// Wraps self in the enum.
    fn as_enum(self) -> content_list::EditorWindowLayoutContent;
    fn as_enum_type(&self) -> content_list::EditorWindowLayoutContentTypeEnum;
    /// In TypePreview mode, this determines the text that is displayed above the preview, in the box that allows users to drag the content "window" around.
    fn as_window_title(&self) -> String;
    /// Used to indicate to the 
    fn data(&mut self) -> &mut EditorWindowLayoutContentData;
    //
    fn get_children(&mut self) -> &mut [EditorWindowLayoutContent];
}
pub struct EditorWindowLayoutContentData {
    pub requests: Vec<EditorWindowLayoutRequest>,
}
impl Default for EditorWindowLayoutContentData { fn default() -> Self { Self {
    requests: Vec::new(),
} } }

pub struct EditorWindowLayoutContentDrawOptions {
    pub force_redraw_due_to_resize: bool,
    pub render_canvas_size: (u32, u32),
    pub my_size_in_pixels: (f32, f32),
    pub assets_manager: AssetsManager,
    pub draw_mode: EditorWindowLayoutContentDrawMode,
    pub general_visibility: f32,
    pub visibility_factors: EditorWindowLayoutContentDrawOptionsVisibilityFactors,
    pub visibility: f32,
}
pub struct EditorWindowLayoutContentDrawOptionsVisibilityFactors {
    pub qvidrunner: f32,
    pub video_properties_editor_tabs: f32,
} impl Default for EditorWindowLayoutContentDrawOptionsVisibilityFactors {
    fn default() -> Self { Self {
        qvidrunner: 1.0,
        video_properties_editor_tabs: 0.0,
} } }
impl EditorWindowLayoutContentDrawOptions {
}
#[derive(Clone)]
pub enum EditorWindowLayoutContentDrawMode {
    Static(EditorWindowLayoutContentSDrawMode),
    /// Represents a simple transition between two DrawModes, where prog=0.0 is the first and prog=1.0 is the second DrawMode. [TODO: Rename this to Transition)]
    Transition {
        modes: [EditorWindowLayoutContentSDrawMode; 2],
        prog: f32,
    },
}
#[derive(Clone)]
#[derive(PartialEq)]
pub enum EditorWindowLayoutContentSDrawMode {
    /// Indicates normal usage.
    Normal,
    /// Show only your type, not your content. This should be fast even when resolution changes repeatedly.
    TypePreview {
        /// If this is set, draw as semi-transparent. This way, whatever is below self can still be seen. This is not normally necessary because, unless the user is moving something around, there should be no overlap.
        moving: bool,
    },
}

pub struct UserInput<'a> {
    pub owned: UserInputOwned<'a>,
    pub clonable: UserInputClonable,
    /// Contains an immutable reference to the current draw's custom actions, and a mutable one to the next custom actions (which will be sent out next frame).
    custom_actions_now: Option<&'a Vec<CustomDrawActions>>,
    custom_actions_next: &'a mut Vec<CustomDrawActions>,
}

pub enum CustomDrawActions {
    /// (false) sets the resolution of all video previews to 0x0 and adds (true) for the next frame. (true) restores the resolution. This causes the background thread to drop its old cache (i have no idea why, but it works, so i don't really care right now)
    VideoPreviewResize(bool),
    /// Some(index) indicates that something is to be edited, while None indicates the opposite.
    SetEditingTo(Option<u32>),
}
impl<'a> UserInput<'a> {
    pub fn new_no_actions(owned: UserInputOwned<'a>, clonable: UserInputClonable, custom_actions_next: &'a mut Vec<CustomDrawActions>) -> UserInput<'a> {
        Self::new(owned, clonable, None, custom_actions_next)
    }
    pub fn new_with_actions(owned: UserInputOwned<'a>, clonable: UserInputClonable, custom_actions_now: &'a Vec<CustomDrawActions>, custom_actions_next: &'a mut Vec<CustomDrawActions>) -> UserInput<'a> {
        Self::new(owned, clonable, Some(custom_actions_now), custom_actions_next)
    }
    pub fn new(owned: UserInputOwned<'a>, clonable: UserInputClonable, custom_actions_now: Option<&'a Vec<CustomDrawActions>>, custom_actions_next: &'a mut Vec<CustomDrawActions>) -> UserInput<'a> {
        Self { owned, clonable, custom_actions_now, custom_actions_next, }
    }
    /// Grants mutable access to the custom actions for the NEXT call to draw. Where get_custom_actions would return None (in non-draw calls), this STILL WORKS. Anything added with this method will be accessible via get_custom_actions only in the NEXT draw event (not this one!)
    pub fn add_custom_action(&mut self, action: CustomDrawActions) {
        self.custom_actions_next.push(action);
    }
    /// Grants immutable access to the custom actions for this call to draw. In InputHandler calls (mouse moved, ...) this is None, as it can only be accessed from draw.
    pub fn get_custom_actions(&self) -> Option<&'a Vec<CustomDrawActions>> {
        self.custom_actions_now
    }
    /// When replacing clonable, always remember to move the old clonable (returned by this function) in a variable and put it back (using this function for a second time) after the nested calls to replace_clonable have returned. This way, if someone decides to call your implementation on handle_input and then tries to do something with the inputs, they do not end up with weird results that are caused by you modifying stuff and not reverting the changes.
    pub fn replace_clonable(&mut self, new: UserInputClonable) -> UserInputClonable { std::mem::replace(&mut self.clonable, new) }
}
pub struct UserInputOwned<'a> {
    pub action: InputAction,
    
    pub project: &'a mut crate::project::Project,

    pub mouse_down_buttons: &'a HashMap<MouseButton, super::MouseButtonDownInfo>,
    pub keyboard_modifiers_state: &'a speedy2d::window::ModifiersState,
    pub keyboard_down_buttons_scancode: &'a HashMap<u32, u8>,
    pub keyboard_down_buttons_virtualkeycode: &'a HashMap<VirtualKeyCode, u8>,
}
#[derive(Clone)]
pub struct UserInputClonable {
    pub mouse_pos: (f32, f32),
}

#[derive(Clone)]
pub enum InputAction {
    None,
    Mouse(MouseAction),
    Keyboard(KeyboardAction),
}
#[derive(Clone)]
pub enum MouseAction {
    Moved,
    ButtonDown(MouseButton),
    ButtonUp(MouseButton),
    Scroll(MouseScrollDistance),
}
#[derive(Clone)]
pub enum KeyboardAction {
    Pressed(KeyScancode, Option<VirtualKeyCode>),
    Released(KeyScancode, Option<VirtualKeyCode>),
    Typed(char),
}
