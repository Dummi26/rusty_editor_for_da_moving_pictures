use crate::{content::content::GenericContentData, project::Project, video::Video};
use std::sync::{Arc, Mutex};

pub struct QuickCommandsHandler {
    sender: std::sync::mpsc::Sender<QctSendable>,
    receiver: std::sync::mpsc::Receiver<QctCompletions>,
    thread: Option<std::thread::JoinHandle<()>>,
    pub completions: Vec<String>,
    project: Project,
}

impl QuickCommandsHandler {
    pub fn exec_command(
        project: &Project,
        command: &str,
        video: &mut Video,
        index: Option<u32>,
    ) -> Result<Vec<QctCommand>, String> {
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
            let vid_type = command_next
                .split_whitespace()
                .next()
                .unwrap()
                .to_lowercase();
            let command_rest = if command_next.len() > vid_type.len() + 1 {
                Some(&command_next[vid_type.len() + 1..])
            } else {
                None
            };
            let gcd = GenericContentData::new(project.clone());
            let vid = match vid_type.as_str() {
                "list" => crate::video::VideoTypeEnum::List(Vec::new()),
                "effect" => crate::video::VideoTypeEnum::WithEffect(
                    Box::new(crate::video::Video::new_full(crate::video::VideoType::new(
                        crate::video::VideoTypeEnum::List(Vec::new()),
                        GenericContentData::new(project.clone()),
                    ))),
                    crate::effect::Effect {
                        effect: crate::effect::effects::EffectsEnum::Nothing(
                            crate::effect::effects::Nothing {},
                        ),
                    },
                ),
                "img" => crate::video::VideoTypeEnum::Image(if let Some(rest) = command_rest {
                    crate::content::image::Image::new(rest.into(), gcd)
                } else {
                    crate::content::image::Image::new("".into(), gcd)
                }),
                "vid" => {
                    crate::video::VideoTypeEnum::Raw(if let Some(rest) = command_rest {
                        match crate::content::input_video::InputVideo::new_from_directory_full_of_frames(rest.into(), (0, 0, true), gcd) {
                            Ok(v) => v,
                            Err(e) => {
                                return Err(format!("Could not create video: {:?}", e));
                            },
                        }
                    } else {
                        crate::content::input_video::InputVideo::new(gcd)
                    })
                }
                "ffmpeg" => crate::video::VideoTypeEnum::Ffmpeg(if let Some(rest) = command_rest {
                    crate::content::ffmpeg_vid::FfmpegVid::new(rest.into(), gcd)
                } else {
                    crate::content::ffmpeg_vid::FfmpegVid::new(std::path::PathBuf::new(), gcd)
                }),
                _ => {
                    return Err(format!(
                        "{} <- expected list/effect/img/vid/ffmpeg",
                        command
                    ))
                }
            };
            return Ok(vec![QctCommand::ApplyChanges(crate::video::VideoChanges {
                video: Some(crate::video::VideoTypeChanges::List(vec![
                    crate::video::VideoTypeChanges_List::Insert(
                        0,
                        crate::video::Video::new_full(crate::video::VideoType::new(
                            vid,
                            GenericContentData::new(project.clone()),
                        )),
                    ),
                ])),
                ..Default::default()
            })]);
        }
        if command.starts_with("wrap") {
            let command_next = &command[5..];
            let wrap_type = command_next
                .split_whitespace()
                .next()
                .unwrap()
                .to_lowercase();
            let command_rest = if command_next.len() > wrap_type.len() + 1 {
                Some(&command_next[wrap_type.len() + 1..])
            } else {
                None
            };
            match wrap_type.as_str() {
                "aspect_ratio" => {
                    if let Some(rest) = command_rest {
                        let mut nums = rest.split(':');
                        if let (Some(n1), Some(n2)) = (nums.next(), nums.next()) {
                            match (n1.parse::<f64>(), n2.parse::<f64>()) {
                                (Ok(n1), Ok(n2)) => {
                                    return Ok(vec![QctCommand::ApplyChanges(
                                        crate::video::VideoChanges {
                                            wrap: Some(
                                                crate::video::VideoChangesWrapWith::AspectRatio(
                                                    crate::curve::CurveData::Constant(n1).into(),
                                                    crate::curve::CurveData::Constant(n2).into(),
                                                ),
                                            ),
                                            ..Default::default()
                                        },
                                    )])
                                }
                                (Ok(_), Err(e2)) => {
                                    return Err(format!(
                                        "width '{}' could not be parsed: {}",
                                        n2, e2
                                    ))
                                }
                                (Err(e1), Ok(_)) => {
                                    return Err(format!(
                                        "height '{}' could not be parsed: {}",
                                        n1, e1
                                    ))
                                }
                                (Err(e1), Err(e2)) => {
                                    return Err(format!(
                                        "width '{}' and height '{}' could not be parsed: {} | {}",
                                        n1, n2, e1, e2
                                    ))
                                }
                            }
                        } else {
                            return Err(format!("wrap type 'aspect_ratio' requires an aspect ratio specified as [width]:[height]."));
                        }
                    }
                }
                "list" => {
                    return Ok(vec![QctCommand::ApplyChanges(crate::video::VideoChanges {
                        wrap: Some(crate::video::VideoChangesWrapWith::List),
                        ..Default::default()
                    })])
                }
                invalid_wrap_type => {
                    return Err(format!("Invalid wrap type: '{}'", invalid_wrap_type))
                }
            }
        }

        Err(format!("{} (?)", command))
    }

    // - - - - - - - - - -

    pub fn request_thread_stop(&self) {
        self.sender.send(QctSendable::Stop).unwrap();
    }
    pub fn wait_for_thread_stop(mut self) {
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
        }
    }
    pub fn request_and_wait_for_thread_stop(self) {
        self.request_thread_stop();
        self.wait_for_thread_stop();
    }
    pub fn refresh(&mut self) -> Vec<QctCommand> {
        let mut cmds = Vec::new();
        while let Ok(recv) = self.receiver.try_recv() {
            match recv {
                QctCompletions::Clear => {
                    self.completions.clear();
                }
                QctCompletions::Set(index, value) => {
                    if self.completions.len() <= index {
                        while self.completions.len() < index {
                            self.completions.push(String::new());
                        }
                        self.completions.push(value);
                    } else {
                        self.completions[index] = value;
                    }
                }
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
        self.sender
            .send(QctSendable::SetEditing(new_editing))
            .unwrap();
    }
    pub fn vid_updated(&self) {
        self.sender.send(QctSendable::VidUpdated).unwrap();
    }
    pub fn exec_query(&self, query_index: Option<usize>) {
        self.sender
            .send(QctSendable::ExecQuery(query_index))
            .unwrap();
    }

    pub fn new(mut edited_part: Option<u32>, project: Project) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let (gen, completions) = std::sync::mpsc::channel();
        let video_access = project.vid();
        let mut s = Self {
            sender,
            receiver: completions,
            completions: Vec::new(),
            project: project.clone(),
            thread: None,
        };
        s.thread = Some(std::thread::spawn(move || {
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
                        }
                        QctSendable::SetEditing(editing2) => {
                            edited_part = editing2;
                            new_editing = true;
                        }
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
                                let exec_result = Self::exec_command(
                                    &project,
                                    command,
                                    &mut *video_access.lock().unwrap(),
                                    edited_part,
                                );
                                match exec_result {
                                    Ok(cmds) => {
                                        if cmds.len() > 0 {
                                            gen.send(QctCompletions::Commands(cmds)).unwrap();
                                        }
                                    }
                                    Err(new_query) => gen
                                        .send(QctCompletions::Command(QctCommand::SetQueryTo(
                                            new_query,
                                        )))
                                        .unwrap(),
                                }
                            }
                        }
                    }
                }
                if new_query || new_editing {
                    // clear
                    new_query = false;
                    gen.send(QctCompletions::Clear).unwrap();
                    possible_commands.clear();
                    processing_command = Some(0);
                }
                if new_editing {
                    new_editing = false;
                    if let Some(editing) = edited_part {
                        let vid = &mut *video_access.lock().unwrap();
                        let editing_part = crate::useful::get_elem_from_index_recursive_mut(
                            vid,
                            &mut editing.clone(),
                        );
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
                            if editing_part_abstract.is_some()
                                && "deselect editing".starts_with(&query)
                            {
                                let s = "deselect editing".to_string();
                                gen.send(QctCompletions::Set(possible_commands.len(), s.clone()))
                                    .unwrap();
                                possible_commands.push(s);
                            }
                        }
                        2 => {
                            if "edit ".starts_with(&query) {
                                let s = "edit [num]".to_string();
                                gen.send(QctCompletions::Set(possible_commands.len(), s.clone()))
                                    .unwrap();
                                possible_commands.push(s);
                            } else if query.starts_with("edit ") {
                                let s = query.clone();
                                gen.send(QctCompletions::Set(possible_commands.len(), s.clone()))
                                    .unwrap();
                                possible_commands.push(s);
                            }
                        }
                        3 => {
                            if editing_part_abstract.is_some() {
                                if "wrap ".starts_with(&query) {
                                    let s = "wrap [mode]".to_string();
                                    gen.send(QctCompletions::Set(
                                        possible_commands.len(),
                                        s.clone(),
                                    ))
                                    .unwrap();
                                    possible_commands.push(s);
                                }
                                if query.starts_with("wrap ") {
                                    let mode = &query[5..];
                                    let mut comps = Vec::new();
                                    if "list".starts_with(mode) && mode.len() <= 4 {
                                        comps.push("list");
                                    }
                                    if "aspect_ratio " == mode {
                                        comps.push("aspect_ratio [w]:[h]");
                                    } else if "aspect_ratio".starts_with(mode) {
                                        comps.push("aspect_ratio");
                                    } else if mode.starts_with("aspect_ratio ") {
                                        comps.push(mode);
                                    }
                                    for comp in comps {
                                        let s = format!("wrap {}", comp);
                                        gen.send(QctCompletions::Set(
                                            possible_commands.len(),
                                            s.clone(),
                                        ))
                                        .unwrap();
                                        possible_commands.push(s);
                                    }
                                }
                            }
                        }
                        4 => {
                            if match editing_part_abstract {
                                EditingPartAbstract::List { .. } => true,
                                _ => false,
                            } {
                                if "add ".starts_with(&query) {
                                    let s = "add [what]".to_string();
                                    gen.send(QctCompletions::Set(
                                        possible_commands.len(),
                                        s.clone(),
                                    ))
                                    .unwrap();
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
                                        let file = if path.len() > last_slash + 1 {
                                            Some(&path[last_slash + 1..])
                                        } else {
                                            None
                                        };
                                        // let mut valid_entries = Vec::new();
                                        if let Ok(dir_entries) =
                                            std::fs::read_dir(std::path::PathBuf::from(dir))
                                        {
                                            for dir_entry in dir_entries {
                                                if let Ok(entry) = dir_entry {
                                                    let file_name_ok = if let Some(file) = file {
                                                        if let Some(file_name) =
                                                            entry.path().file_name()
                                                        {
                                                            file_name
                                                                .to_string_lossy()
                                                                .to_string()
                                                                .starts_with(file)
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        true
                                                    };
                                                    if file_name_ok {
                                                        suggestions.push(format!(
                                                            "img {}",
                                                            entry.path().to_string_lossy()
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if "vid".starts_with(whatl) {
                                        suggestions.push("vid".to_string());
                                    } // no else!
                                    if whatl == "vid" {
                                        suggestions.push("vid [path]".to_string());
                                    }
                                    if whatl.starts_with("vid ") {
                                        // suggestions.push(what.to_string());
                                        let path = &what[4..];
                                        let last_slash = path.rfind("/").unwrap_or(0);
                                        let dir = &path[..last_slash];
                                        let file = if path.len() > last_slash + 1 {
                                            Some(&path[last_slash + 1..])
                                        } else {
                                            None
                                        };
                                        // let mut valid_entries = Vec::new();
                                        if let Ok(dir_entries) =
                                            std::fs::read_dir(std::path::PathBuf::from(dir))
                                        {
                                            for dir_entry in dir_entries {
                                                if let Ok(entry) = dir_entry {
                                                    let file_name_ok = if let Some(file) = file {
                                                        if let Some(file_name) =
                                                            entry.path().file_name()
                                                        {
                                                            file_name
                                                                .to_string_lossy()
                                                                .to_string()
                                                                .starts_with(file)
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        true
                                                    };
                                                    if file_name_ok {
                                                        suggestions.push(format!(
                                                            "vid {}",
                                                            entry.path().to_string_lossy()
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if "ffmpeg".starts_with(whatl) {
                                        suggestions.push("ffmpeg".to_string());
                                    }
                                    if whatl == "ffmpeg" {
                                        suggestions.push("ffmpeg [path]".to_string());
                                    }
                                    if whatl.starts_with("ffmpeg ") {
                                        // suggestions.push(what.to_string());
                                        let path = &what[7..];
                                        let last_slash = path.rfind("/").unwrap_or(0);
                                        let dir = &path[..last_slash];
                                        let file = if path.len() > last_slash + 1 {
                                            Some(&path[last_slash + 1..])
                                        } else {
                                            None
                                        };
                                        // let mut valid_entries = Vec::new();
                                        if let Ok(dir_entries) =
                                            std::fs::read_dir(std::path::PathBuf::from(dir))
                                        {
                                            for dir_entry in dir_entries {
                                                if let Ok(entry) = dir_entry {
                                                    let file_name_ok = if let Some(file) = file {
                                                        if let Some(file_name) =
                                                            entry.path().file_name()
                                                        {
                                                            file_name
                                                                .to_string_lossy()
                                                                .to_string()
                                                                .starts_with(file)
                                                        } else {
                                                            false
                                                        }
                                                    } else {
                                                        true
                                                    };
                                                    if file_name_ok {
                                                        suggestions.push(format!(
                                                            "ffmpeg {}",
                                                            entry.path().to_string_lossy()
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    for suggestion in suggestions {
                                        let s = format!("add {}", suggestion);
                                        gen.send(QctCompletions::Set(
                                            possible_commands.len(),
                                            s.clone(),
                                        ))
                                        .unwrap();
                                        possible_commands.push(s);
                                    }
                                }
                            }
                        }
                        _ => {
                            if "test".starts_with(&query) {
                                let s = "test".to_string();
                                gen.send(QctCompletions::Set(possible_commands.len(), s.clone()))
                                    .unwrap();
                                possible_commands.push(s);
                            }

                            processing_command = None;
                        }
                    }
                }
            }
        }));
        s
    }
}

enum EditingPartAbstract {
    None,
    List {
        length: usize,
    },
    AspectRatio {
        vid: Box<Self>,
        width: crate::curve::Curve,
        height: crate::curve::Curve,
    },
    WithEffect {
        effect: (),
        contained: Box<Self>,
    },
    Text(crate::content::text::TextType),
    Image {
        path: std::path::PathBuf,
    },
    Video {
        path: std::path::PathBuf,
    },
    Ffmpeg {
        path: std::path::PathBuf,
    },
}
impl From<&crate::video::Video> for EditingPartAbstract {
    fn from(vid: &crate::video::Video) -> Self {
        match &vid.video.vt {
            crate::video::VideoTypeEnum::List(vec) => Self::List { length: vec.len() },
            crate::video::VideoTypeEnum::AspectRatio(v, w, h) => Self::AspectRatio {
                vid: Box::new(v.as_ref().into()),
                width: w.clone(),
                height: h.clone(),
            },
            crate::video::VideoTypeEnum::WithEffect(contained, _effect) => Self::WithEffect {
                effect: (), /* TODO */
                contained: Box::new(contained.as_ref().into()),
            },
            crate::video::VideoTypeEnum::Text(t) => Self::Text(t.text().clone()),
            crate::video::VideoTypeEnum::Image(img) => Self::Image {
                path: img.path().clone(),
            },
            crate::video::VideoTypeEnum::Raw(vid) => Self::Video {
                path: vid.get_dir().clone(),
            },
            crate::video::VideoTypeEnum::Ffmpeg(vid) => Self::Ffmpeg {
                path: vid.path().clone(),
            },
        }
    }
}
impl EditingPartAbstract {
    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }
}

impl std::ops::Drop for QuickCommandsHandler {
    fn drop(&mut self) {
        self.request_thread_stop();
    }
}

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
