use std::sync::{Arc, Mutex};

use crate::{video::{Video, VideoChanges}, content::content::Content, cli::Clz, useful};

use super::{EditorWindowHandler, content_list::{EditorWindowLayoutContent, EditorWindowLayoutContentEnum, EditorWindowLayoutContentTypeEnum}, layout::EditorWindowLayoutContentTrait, content::placeholder::Placeholder};

pub enum EditorWindowLayoutRequest {
    TypePreviewModeBecomeDraggedWindow { size: (f32, f32), grab_position: (f32, f32), take: bool, },
    ChangeMeTo(EditorWindowLayoutContentTypeEnum),
    SelectForEditing(u32),
    DeselectForEditing,
    EditingChangesApply(VideoChanges),
    /// Like EditingChangesApply, but for when changes were applied by directly accessing the Video object through the Arc<Mutex<_>>. Unlike EditingChangesApply, this is not limited to editing the part that is selected as the one to be edited.
    AppliedChangesToVideo,
}

impl EditorWindowHandler {
    pub fn handle_requests(&mut self) {
        RequestActions::new(self)
            .handle_request(&mut self.layout)
            .apply(self);
    }
}

impl RequestActions {
    fn handle_request_(&mut self, content: &mut EditorWindowLayoutContent) {
        // println!("Handle Request: {}", content.as_window_title());
        for child in content.get_children() {
            self.handle_request_(child);
        };
        let requests = std::mem::replace(&mut content.data().requests, Vec::new());
        for request in requests {
            match request {
                EditorWindowLayoutRequest::TypePreviewModeBecomeDraggedWindow { size, grab_position, take } => {
                    if !self.dragged_window_already_set {
                        self.dragged_window_already_set = true;
                        if take || self.dragged_window.is_some() {
                            let replace_with = if let Some(dragged) = self.dragged_window.take() {
                                dragged.0.as_enum()
                            } else {
                                Placeholder::new().as_enum()
                            };
                            match content.c { // the ones that can't be replaced (they will consume the dragged window)
                                EditorWindowLayoutContentEnum::SpecialQVidRunner(_) => (),
                                _ => {
                                    let content = std::mem::replace(content, replace_with);
                                    match content.c { // the ones that can be replaced without becoming a new dragged window
                                        EditorWindowLayoutContentEnum::Placeholder(_) => (),
                                        _ => self.dragged_window = Some((content, size, grab_position)),
                                    };
                                },
                            };
                        };
                    };
                },
                EditorWindowLayoutRequest::ChangeMeTo(new) => {
                    let new = match new {
                        EditorWindowLayoutContentTypeEnum::Placeholder => None,
                        EditorWindowLayoutContentTypeEnum::VideoPreview => None,
                        EditorWindowLayoutContentTypeEnum::VideoTree => Some(crate::gui::speedy2d::content::video_tree::VideoTree::new(self.video.clone()).as_enum()),
                        EditorWindowLayoutContentTypeEnum::VideoPropertiesEditor => Some(crate::gui::speedy2d::content::video_properties_editor::VideoPropertiesEditor::new(self.video.clone()).as_enum()),
                        EditorWindowLayoutContentTypeEnum::LayoutHalf => Some(crate::gui::speedy2d::content::layout::half::Half::new_placeholders(false, 0.5).as_enum()),
                        EditorWindowLayoutContentTypeEnum::SpecialQVidRunner => None,
                    };
                    if let Some(new) = new { *content = new; };
                },
                EditorWindowLayoutRequest::SelectForEditing(path) => /*if let None = self.edited_part*/ {
                    if self.edited_part != Some(path) {
                        self.edited_part = Some(path);
                        self.edited_path_was_changed = true;
                    };
                },
                EditorWindowLayoutRequest::DeselectForEditing => {
                    if self.edited_part != None {
                        self.edited_part = None;
                        self.edited_path_was_changed = true;
                    };
                },
                EditorWindowLayoutRequest::EditingChangesApply(changes) => if let Some(index) = self.edited_part {
                    let actual_vid = &mut *self.video.lock().unwrap();
                    // Follow path and set actual_vid to the result
                    if let Some(actual_vid) = useful::get_elem_from_index_recursive_mut(actual_vid, &mut index.clone()) {
                        actual_vid.as_content_changes = changes;
                        if actual_vid.apply_changes() {
                            println!("{}", Clz::progress("Applied changes successfully."));
                        } else {
                            println!("{}", Clz::progress("Did not apply changes."));
                        };
                        self.edited_path_was_changed = true;
                        self.edited_part_requires_update = true;
                    } else {
                        panic!("\n{}\n{}{}{}\n",
                            Clz::error_info("While attempting to locate video part to apply changes to, experienced an out-of-index fault!"),
                            Clz::error_details("Index was "), Clz::error_cause(format!("{}", index).as_str()), Clz::error_details("."),
                        );
                    };
                } else {
                    println!("{}", Clz::undecided("Attempted to apply some changes to a video part, but no part is selected for editing."));
                },
                EditorWindowLayoutRequest::AppliedChangesToVideo => todo!(),
            };
        };
    }



    pub fn new(container: &mut EditorWindowHandler) -> Self {
        Self {
            video: container.project.vid.get_vid_mutex_arc(),
            dragged_window: container.dragged_window.take(),
            dragged_window_already_set: false,
            edited_part: container.edited_part.take(),
            edited_path_was_changed: false,
            edited_part_requires_update: false,
        }
    }

    pub fn handle_request(mut self, content: &mut EditorWindowLayoutContent) -> Self {
        self.handle_request_(content);
        self
    }

    pub fn apply(self, container: &mut EditorWindowHandler) {
        container.dragged_window = self.dragged_window;
        container.edited_part = self.edited_part;
        if self.edited_path_was_changed {
            container.custom_actions.push(super::layout::CustomDrawActions::SetEditingTo(container.edited_part));
        };
        if self.edited_part_requires_update {
            container.custom_actions.push(super::layout::CustomDrawActions::VideoPreviewResize(false));
        };
    }
}

struct RequestActions {
    pub video: Arc<Mutex<Video>>,
    pub dragged_window: Option<(EditorWindowLayoutContent, (f32, f32), (f32, f32))>,
    pub dragged_window_already_set: bool,
    pub edited_part: Option<u32>,
    pub edited_path_was_changed: bool,
    pub edited_part_requires_update: bool,
}
