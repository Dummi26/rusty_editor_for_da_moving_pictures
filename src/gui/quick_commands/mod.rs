use std::sync::{Arc, Mutex};
use crate::video::Video;

pub struct QuickCommandsHandler {
    sender: std::sync::mpsc::Sender<QctSendable>,
    receiver: std::sync::mpsc::Receiver<QctCompletions>,
    thread: Option<std::thread::JoinHandle<()>>,
    pub completions: Vec<String>,
}

impl QuickCommandsHandler {
    pub fn exec_command(command: &str, video: &mut Video, index: Option<u32>) -> Result<Vec<QctCommand>, String> {
        if command == "deselect editing" {
            return Ok(vec![QctCommand::UnsetEditing]);
        }
        {
            let mut split = command.split_whitespace();
            if split.next() == Some("edit") {
                if let Some(num) = split.next() {
                    if let Ok(num) = num.parse() {
                        return Ok(vec![QctCommand::SetEditingTo(num)]);
                    } else {
                        return Err(format!("{} <- not a number", command));
                    }
                } else {
                    return Err(format!("edit [num]"));
                }
            }
        }
        if command.starts_with("add ") {
            let command_next = &command[4..];
            let vid_type = command_next.split_whitespace().next().unwrap().to_lowercase();
            let command_rest = if command_next.len() > vid_type.len() + 1 { Some(&command_next[vid_type.len()+1..]) } else { None };
            let vid = match vid_type.as_str() {
                "list" => crate::video::VideoTypeEnum::List(
                    Vec::new()
                ),
                "effect" => crate::video::VideoTypeEnum::WithEffect(
                    Box::new(crate::video::Video::new_full(crate::video::VideoType::new(crate::video::VideoTypeEnum::List(Vec::new())))),
                    crate::effect::Effect { effect: crate::effect::effects::EffectsEnum::Nothing(crate::effect::effects::Nothing {}) }
                ),
                "img" => crate::video::VideoTypeEnum::Image(
                    if let Some(rest) = command_rest {
                        crate::content::image::Image::new(rest.into())
                    } else {
                        crate::content::image::Image::new("".into())
                    }
                ),
                "vid" => crate::video::VideoTypeEnum::Raw(
                    if let Some(rest) = command_rest {
                        match crate::content::input_video::InputVideo::new_from_directory_full_of_frames(rest.into(), (0, 0, true)) {
                            Ok(v) => v,
                            Err(e) => {
                                return Err(format!("Could not create video: {:?}", e));
                            },
                        }
                    } else {
                        crate::content::input_video::InputVideo::new()
                    }
                ),
                _ => return Err(format!("{} <- expected list/effect/img/vid", command)),
            };
            return Ok(
                vec![
                    QctCommand::ApplyChanges(
                        crate::video::VideoChanges {
                            video: Some(
                                crate::video::VideoTypeChanges::List(
                                    vec![
                                        crate::video::VideoTypeChanges_List::Insert(
                                            0,
                                            crate::video::Video::new_full(
                                                crate::video::VideoType::new(
                                                    vid
                                                )
                                            )
                                        )
                                    ]
                                )
                            ),
                            ..Default::default()
                        }
                    )
                ]
            );
        }

        Err(format!("{} (?)", command))
    }

    // - - - - - - - - - -

    pub fn request_thread_stop(&self) { self.sender.send(QctSendable::Stop).unwrap(); }
    pub fn wait_for_thread_stop(mut self) { if let Some(t) = self.thread.take() { t.join().unwrap(); } }
    pub fn request_and_wait_for_thread_stop(self) { self.request_thread_stop(); self.wait_for_thread_stop(); }
    pub fn refresh(&mut self) -> Vec<QctCommand> {
        let mut cmds = Vec::new();
        while let Ok(recv) = self.receiver.try_recv() {
            match recv {
                QctCompletions::Clear => {
                    self.completions.clear();
                },
                QctCompletions::Set(index, value) => {
                    if self.completions.len() <= index {
                        while self.completions.len() < index {
                            self.completions.push(String::new());
                        }
                        self.completions.push(value);
                    } else {
                        self.completions[index] = value;
                    }
                },
                QctCompletions::Command(cmd) => cmds.push(cmd),
                QctCompletions::Commands(cmd) => cmds.extend(cmd.into_iter()),
            }
        }
        cmds
    }
    pub fn set_new_query(&self, new_query: String) {
        self.sender.send(QctSendable::SetQuery(new_query)).unwrap();
    }
    pub fn set_editing(&self, new_editing: Option<u32>) {
        self.sender.send(QctSendable::SetEditing(new_editing)).unwrap();
    }
    pub fn vid_updated(&self) {
        self.sender.send(QctSendable::VidUpdated).unwrap();
    }
    pub fn exec_query(&self, query_index: Option<usize>) {
        self.sender.send(QctSendable::ExecQuery(query_index)).unwrap();
    }

    pub fn new(mut edited_part: Option<u32>, video_access: Arc<Mutex<Video>>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let (gen, completions) = std::sync::mpsc::channel();
        Self {
            sender,
            receiver: completions,
            completions: Vec::new(),
            thread: Some(std::thread::spawn(move || {
                let mut possible_commands = Vec::<String>::new();
                let mut processing_command = None;
                let mut query = String::new();
                let mut editing_part_abstract = EditingPartAbstract::None;
                let mut new_query = false;
                let mut new_editing = true;
                'l: loop {
                    // Receiving
                    while let Ok(recv) = receiver.try_recv() {
                        match recv {
                            QctSendable::Stop => break 'l,
                            QctSendable::SetQuery(query2) => {
                                query = query2;
                                new_query = true;
                            },
                            QctSendable::SetEditing(editing2) => {
                                edited_part = editing2;
                                new_editing = true;
                            },
                            QctSendable::VidUpdated => new_editing = true,
                            QctSendable::ExecQuery(query_index) => {
                                let command = if let Some(query_index) = query_index {
                                    if let Some(command) = possible_commands.get(query_index) {
                                        Some(command.as_str())
                                    } else {
                                        None
                                    }
                                } else {
                                    Some(query.as_str())
                                };
                                if let Some(command) = command {
                                    let exec_result = Self::exec_command(command, &mut *video_access.lock().unwrap(), edited_part);
                                    match exec_result {
                                        Ok(cmds) => if cmds.len() > 0 {
                                            gen.send(QctCompletions::Commands(cmds)).unwrap();
                                        },
                                        Err(new_query) => gen.send(QctCompletions::Command(QctCommand::SetQueryTo(new_query))).unwrap(),
                                    }
                                }
                            },
                        }
                    }
                    if new_query || new_editing { // clear
                        new_query = false;
                        gen.send(QctCompletions::Clear).unwrap(); possible_commands.clear(); processing_command = Some(0); }
                    if new_editing {
                        new_editing = false;
                        if let Some(editing) = edited_part {
                            let vid = &mut *video_access.lock().unwrap();
                            let editing_part = crate::useful::get_elem_from_index_recursive_mut(vid, &mut editing.clone());
                            if let Some(editing_part) = editing_part {
                                editing_part_abstract = (&*editing_part).into();
                            } else {
                                editing_part_abstract = EditingPartAbstract::None;
                            }
                        } else {
                            editing_part_abstract = EditingPartAbstract::None;
                        }
                    }

                    // Processing
                    if let Some(pc) = &mut processing_command {
                        *pc += 1;
                        match pc {
                            1 => {
                                if editing_part_abstract.is_some() && "deselect editing".starts_with(&query) {
                                    let s = "deselect editing".to_string();
                                    gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                    possible_commands.push(s);
                                }
                            },
                            2 => {
                                if "edit ".starts_with(&query) {
                                    let s = "edit [num]".to_string();
                                    gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                    possible_commands.push(s);
                                } else if query.starts_with("edit ") {
                                    let s = query.clone();
                                    gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                    possible_commands.push(s);
                                }
                            },
                            3 => if match editing_part_abstract { EditingPartAbstract::List {..} => true, _ => false } {
                                if "add ".starts_with(&query) {
                                    let s = "add [what]".to_string();
                                    gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                    possible_commands.push(s);
                                }
                                if query.starts_with("add ") {
                                    let what = query[4..].to_string();
                                    let whatl = what.to_lowercase();
                                    let whatl = whatl.as_str();
                                    let mut suggestions = Vec::new();
                                    if "list".starts_with(whatl) {
                                        suggestions.push("list".to_string());
                                    }
                                    if "effect".starts_with(whatl) {
                                        suggestions.push("effect".to_string());
                                    }
                                    if "img".starts_with(whatl) {
                                        suggestions.push("img".to_string());
                                    }
                                    if whatl == "img" {
                                        suggestions.push("img [path]".to_string());
                                    }
                                    if whatl.starts_with("img ") {
                                        // suggestions.push(what.to_string());
                                        let path = &what[4..];
                                        let last_slash = path.rfind("/").unwrap_or(0);
                                        let dir = &path[..last_slash];
                                        let file = if path.len() > last_slash+1 { Some(&path[last_slash+1..]) } else { None };
                                        // let mut valid_entries = Vec::new();
                                        if let Ok(dir_entries) = std::fs::read_dir(std::path::PathBuf::from(dir)) {
                                            for dir_entry in dir_entries {
                                                if let Ok(entry) = dir_entry {
                                                    let file_name_ok = if let Some(file) = file {
                                                        if let Some(file_name) = entry.path().file_name() {
                                                            file_name.to_string_lossy().to_string().starts_with(file)
                                                        } else { false }
                                                    } else { true };
                                                    if file_name_ok {
                                                        suggestions.push(format!("img {}", entry.path().to_string_lossy()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if "vid".starts_with(whatl) {
                                        suggestions.push("vid".to_string());
                                    } // no else!
                                    if whatl.starts_with("vid") {
                                        suggestions.push("vid [path]".to_string());
                                    }
                                    for suggestion in suggestions {
                                        let s = format!("add {}", suggestion);
                                        gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                        possible_commands.push(s);
                                    }
                                }
                            },
                            _ => {
                                if "test".starts_with(&query) {
                                    let s = "test".to_string();
                                    gen.send(QctCompletions::Set(possible_commands.len(), s.clone())).unwrap();
                                    possible_commands.push(s);
                                }

                                processing_command = None;
                            },
                        }
                    }
                }
            })),
        }
    }
}


enum EditingPartAbstract {
    None,
    List { length: usize, },
    WithEffect { effect: (), contained: Box<Self>, },
    Image { path: std::path::PathBuf, },
    Video { path: std::path::PathBuf, },
}
impl From<&crate::video::Video> for EditingPartAbstract { fn from(vid: &crate::video::Video) -> Self {
    match &vid.video.vt {
        crate::video::VideoTypeEnum::List(vec) => Self::List { length: vec.len(), },
        crate::video::VideoTypeEnum::WithEffect(contained, _effect) => Self::WithEffect { effect: () /* TODO */, contained: Box::new(contained.as_ref().into()), },
        crate::video::VideoTypeEnum::Image(img) => Self::Image { path: img.path().clone(), },
        crate::video::VideoTypeEnum::Raw(vid) => Self::Video { path: vid.get_dir().clone() },
    }
} }
impl EditingPartAbstract {
    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }
}

impl std::ops::Drop for QuickCommandsHandler { fn drop(&mut self) { self.request_thread_stop(); } }

/// QuickCommand thread sendable
enum QctSendable {
    Stop,
    SetQuery(String),
    SetEditing(Option<u32>),
    VidUpdated,
    /// If None, executes the query, if Some(n), executes the nth autocompletion.
    ExecQuery(Option<usize>),
}

enum QctCompletions {
    Clear,
    Set(usize, String),
    Command(QctCommand),
    Commands(Vec<QctCommand>),
}

pub enum QctCommand {
    UnsetEditing,
    SetEditingTo(u32),
    SetQueryTo(String),
    ApplyChanges(crate::video::VideoChanges),
}