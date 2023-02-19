use speedy2d::{
    color::Color,
    dimen::Vector2,
    font::{TextLayout, TextOptions},
};

use crate::{
    gui::speedy2d::{
        content_list::EditorWindowLayoutContentEnum,
        layout::{EditorWindowLayoutContentData, EditorWindowLayoutContentTrait},
    },
    project::Project,
    useful,
};

pub enum QVidRunnerMode {
    /// The user opened QVidRunner (probably through a keyboard shortcut)
    Normal,
}

pub struct QVidRunner {
    mode: QVidRunnerMode,
    pub query: String,
    /// This is set while the user is going through the suggestions using TAB. This being Some(_) also indicates that the background thread's query is NOT updated (the thread has real_query instead of query if real_query is set)
    pub real_query: Option<(String, usize)>,
    command_handler: crate::gui::quick_commands::QuickCommandsHandler,
    layout_content_data: EditorWindowLayoutContentData,
}

impl EditorWindowLayoutContentTrait for QVidRunner {
    fn was_changed_custom(&self) -> bool {
        todo!()
    }

    fn draw_onto_custom(
        &mut self,
        draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions,
        graphics: &mut speedy2d::Graphics2D,
        position: &(f32, f32, f32, f32),
        input: &mut crate::gui::speedy2d::layout::UserInput,
    ) {
        let font = draw_opts.assets_manager.get_default_font();
        let text = font.layout_text(self.query.as_str(), position.3 * 0.75, TextOptions::new());
        let rx = (position.2 - text.width()) / 2.0;
        graphics.draw_text(
            Vector2::new(position.0 + rx, position.1),
            Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            &text,
        );
        let line_height = position.3 * 0.25;
        for command in self.command_handler.refresh() {
            match command {
                crate::gui::quick_commands::QctCommand::UnsetEditing => self.data().requests.push(
                    crate::gui::speedy2d::request::EditorWindowLayoutRequest::DeselectForEditing,
                ),
                crate::gui::quick_commands::QctCommand::SetEditingTo(i) => {
                    self.data().requests.push(
                        crate::gui::speedy2d::request::EditorWindowLayoutRequest::SelectForEditing(
                            i,
                        ),
                    )
                }
                crate::gui::quick_commands::QctCommand::SetQueryTo(new_query) => {
                    self.query = new_query
                }
                crate::gui::quick_commands::QctCommand::ApplyChanges(changes) => {
                    println!("Applying changes!");
                    // input.add_custom_action(crate::gui::speedy2d::layout::CustomDrawActions::)
                    self.data().requests.push(crate::gui::speedy2d::request::EditorWindowLayoutRequest::EditingChangesApply(changes));
                }
            }
        }
        for (index, text) in self.command_handler.completions.iter().enumerate() {
            let text = font.layout_text(text, line_height * 0.75, TextOptions::new());
            graphics.draw_text(
                Vector2::new(
                    position.0,
                    position.1 + position.3 + index as f32 * line_height,
                ),
                Color::from_rgba(0.8, 0.8, 0.8, 0.8),
                &text,
            );
        }
        if let Some(actions) = input.get_custom_actions() {
            for action in actions {
                match action {
                    crate::gui::speedy2d::layout::CustomDrawActions::SetVideoPreviewActive(_) => {}
                    crate::gui::speedy2d::layout::CustomDrawActions::SetEditingTo(editing) => {
                        self.command_handler.set_editing(*editing)
                    }
                    crate::gui::speedy2d::layout::CustomDrawActions::ChangedVideo => {
                        self.command_handler.vid_updated()
                    }
                }
            }
        }
    }

    fn handle_input_custom(
        &mut self,
        draw_opts: &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentDrawOptions,
        input: &mut crate::gui::speedy2d::layout::UserInput,
    ) {
        match &input.owned.action {
            crate::gui::speedy2d::layout::InputAction::None
            | crate::gui::speedy2d::layout::InputAction::Mouse(_) => (),
            crate::gui::speedy2d::layout::InputAction::Keyboard(action) => match action {
                crate::gui::speedy2d::layout::KeyboardAction::Pressed(_, _)
                | crate::gui::speedy2d::layout::KeyboardAction::Released(_, _) => (),
                crate::gui::speedy2d::layout::KeyboardAction::Typed(ch) => {
                    if !(input.owned.keyboard_modifiers_state.logo()
                        || input.owned.keyboard_modifiers_state.ctrl()
                        || input.owned.keyboard_modifiers_state.alt())
                    {
                        let mut exec = false;
                        let changed = match useful::CharOrAction::from(ch) {
                            useful::CharOrAction::Char(ch) => {
                                self.query.push(ch.clone());
                                true
                            }
                            useful::CharOrAction::Enter => {
                                if self.query.len() != 0 {
                                    exec = true;
                                    self.query.clear();
                                    true
                                } else {
                                    false
                                }
                            }
                            useful::CharOrAction::Delete | useful::CharOrAction::Backspace => {
                                self.query = {
                                    let mut vec = Vec::from_iter(self.query.chars());
                                    vec.pop();
                                    String::from_iter(vec.into_iter())
                                };
                                true
                            }
                            useful::CharOrAction::Tab => {
                                if !self.command_handler.completions.is_empty() {
                                    let mut revert_real_query = false;
                                    if let Some((_, completions_index)) = &mut self.real_query {
                                        if input.owned.keyboard_modifiers_state.shift() == false {
                                            *completions_index += 1;
                                            if *completions_index
                                                >= self.command_handler.completions.len()
                                            {
                                                revert_real_query = true;
                                            }
                                        } else {
                                            if *completions_index == 0 {
                                                revert_real_query = true;
                                            } else {
                                                *completions_index -= 1;
                                            }
                                        }
                                        if !revert_real_query {
                                            self.query = self.command_handler.completions
                                                [*completions_index]
                                                .clone();
                                        }
                                    } else {
                                        let i = if input.owned.keyboard_modifiers_state.shift() {
                                            self.command_handler.completions.len() - 1
                                        } else {
                                            0
                                        };
                                        self.real_query = Some((
                                            std::mem::replace(
                                                &mut self.query,
                                                self.command_handler.completions[i].clone(),
                                            ),
                                            i,
                                        ));
                                    }
                                    if revert_real_query {
                                        if let Some((real_query, _)) = self.real_query.take() {
                                            self.query = real_query;
                                        }
                                    }
                                }
                                false
                            }
                            useful::CharOrAction::Esc => false,
                            useful::CharOrAction::Ignored => false,
                        };
                        if exec {
                            // exec BEFORE setting new query!
                            self.command_handler.exec_query(
                                if let Some((_, index)) = self.real_query {
                                    Some(index)
                                } else {
                                    None
                                },
                            );
                        }
                        if changed {
                            self.real_query = None;
                            self.command_handler.set_new_query(self.query.clone());
                        }
                    };
                }
            },
        };
    }

    fn as_enum(self) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContent {
        EditorWindowLayoutContentEnum::SpecialQVidRunner(self).into()
    }
    fn as_enum_type(
        &self,
    ) -> crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum {
        crate::gui::speedy2d::content_list::EditorWindowLayoutContentTypeEnum::SpecialQVidRunner
    }

    fn as_window_title(&self) -> String {
        "QVidRunner".to_string()
    }

    fn data(&mut self) -> &mut crate::gui::speedy2d::layout::EditorWindowLayoutContentData {
        &mut self.layout_content_data
    }

    fn get_children(
        &mut self,
    ) -> &mut [crate::gui::speedy2d::content_list::EditorWindowLayoutContent] {
        &mut []
    }
}

impl QVidRunner {
    pub fn new(mode: QVidRunnerMode, selected: Option<u32>, proj: Project) -> Self {
        Self {
            mode,
            query: String::new(),
            real_query: None,
            command_handler: crate::gui::quick_commands::QuickCommandsHandler::new(selected, proj),
            layout_content_data: EditorWindowLayoutContentData::default(),
        }
    }
}
