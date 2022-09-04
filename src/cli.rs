use std::{path::PathBuf};

pub mod Clz {
    use colored::{Colorize, ColoredString};

    /// Use this when none of the other options match. Exclude this in the final build - it is only a placeholder!
    pub fn undecided<C>(clz: C) -> ColoredString where C: Colorize { clz.black().on_cyan() }
    
    /// Indicates that a task will start after this message has been sent out.
    pub fn starting<C>(clz: C) -> ColoredString where C: Colorize { clz.green() }
    /// Gives status updates for a task. This style is used for messages leading up to completed. It is also used for tasks that do not work towards any goal (like guis) to send out status messages.
    pub fn progress<C>(clz: C) -> ColoredString where C: Colorize { clz.yellow() }
    /// Indicates that something has successfully completed its task.
    pub fn completed<C>(clz: C) -> ColoredString where C: Colorize { clz.green() }
    /// Gives further information on a task's completion. This is normally used right after completed, to give more information that are not necessarily important to the user.
    pub fn completed_info<C>(clz: C) -> ColoredString where C: Colorize { clz.green() }

    /// Indicates that something has gone wrong. An error is fatal. Use this to describe where, when, and maybe why the error has happened. Usually combined with the other error_* options.
    pub fn error_info<C>(clz: C) -> ColoredString where C: Colorize { clz.blue() }
    /// This represents the cause of an error. If it was user input, for example, the error-causing part of said input should be in this style.
    pub fn error_cause<C>(clz: C) -> ColoredString where C: Colorize { clz.red() }
    /// This represents error details. It is mostly used to color the string representation of actual 'std::error::Error's.
    pub fn error_details<C>(clz: C) -> ColoredString where C: Colorize { clz.magenta() }
}

#[derive(Default)]
pub struct CustomArgs {
    pub project_path: Option<PathBuf>,
    pub action: Option<Action>,
    pub export_options: Option<crate::video_export_settings::VideoExportSettings>,
}

#[derive(Debug)]
pub enum Action {
    OpenProjectInGui,
    ExportProjectToFrames,
    Exit,
}
impl std::fmt::Display for Action { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } }

impl CustomArgs {
    fn parse_arg(&mut self, arg: Vec<String>) {
        match arg.first().unwrap().as_str() {
            "proj-path" => match arg.len() - 1 {
                1 => self.project_path = Some(PathBuf::from(&arg[1])),
                _ => panic!("\n{}\n",
                    Clz::error_info("proj-path requires exactly one argument: --proj-path [path],"),
                ),
            },
            "action" => match arg.len() - 1 {
                1 => self.action = Some(match arg[1].as_str() {
                    "OpenProjectInGui" => Action::OpenProjectInGui,
                    "ExportProjectToFrames" => Action::ExportProjectToFrames,
                    ac => panic!("\n{}{}{}\n    {}\n    {}\n",
                        Clz::error_info("Invalid action '"), Clz::error_cause(ac), Clz::error_info("'! [action] in --action [æction] may only be one of the following:"),
                        Clz::undecided("OpenProjectInGui"),
                        Clz::undecided("ExportProjectToFrames"),
                    ),
                }),
                _ => panic!("\n{}\n",
                    Clz::error_info("action requires one argument: --action [action]."),
                ),
            }
            "export-options" => match arg.len() - 1 {
                4 => self.export_options = Some(crate::video_export_settings::VideoExportSettings {
                    output_path: PathBuf::from(arg[1].as_str()),
                    width: match arg[2].parse() {
                        Ok(v) => v,
                        Err(err) => panic!("\n{}\n{}{}{}{}\n",
                            Clz::error_info("Could not read [width] in --export-options [] [width] [] []:"),
                            Clz::error_info("Could not parse '"), Clz::error_cause(arg[2].as_str()), Clz::error_info("' into an integer: "), Clz::error_details(err.to_string().as_str()),
                        ),
                    },
                    height: match arg[3].parse() {
                        Ok(v) => v,
                        Err(err) => panic!("\n{}\n{}{}{}{}\n",
                            Clz::error_info("Could not read [height] in --export-options [] [] [height] []:"),
                            Clz::error_info("Could not parse '"), Clz::error_cause(arg[2].as_str()), Clz::error_info("' into an integer: "), Clz::error_details(err.to_string().as_str()),
                        ),
                    },
                    frames: match arg[4].parse() {
                        Ok(v) => v,
                        Err(err) => panic!("\n{}\n{}{}{}{}\n",
                            Clz::error_info("Could not read [frames] in --export-options [] [] [] [frames]:"),
                            Clz::error_info("Could not parse '"), Clz::error_cause(arg[2].as_str()), Clz::error_info("' into an integer: "), Clz::error_details(err.to_string().as_str()),
                        ),
                    },
                }),
                _ => panic!("\n{}\n",
                    Clz::error_info("export-options requires 4 arguments: --export-options [output path] [width] [height] [frames]"),
                ),
            }
            invalid_arg => panic!("\n{} {} {}\n    {} {}\n    {} {}\n{}\n",
                Clz::error_info("--arg"), Clz::error_cause(invalid_arg), Clz::error_info("is invalid. Valid args are:"),
                Clz::error_info("proj-path"), Clz::error_info("[path]"),
                Clz::error_info("action"), Clz::error_info("[action]"),
                Clz::error_info("To use these: --[arg] [...], for example: '--proj-path \"/path/to/file.txt\"'."),
            ),
        };
    }
}



impl CustomArgs {
    pub fn read_from_env() -> Self {
        // read args into a more easy-to-parse format
        let args = {
            let mut args = Vec::new();
            let mut args_current = Vec::new();
            for arg in std::env::args() {
                if arg.starts_with("--") {
                    if args_current.len() > 0 {
                        args.push(args_current);
                    };
                    args_current = vec![arg[2..].to_string()];
                } else {
                    args_current.push(arg);
                };
            };
            if args_current.len() > 0 { args.push(args_current); };
            if let Some(first) = args.first() {
                if ! first.first().expect("a 0-length args_current should never be pushed to args!").starts_with("--") {
                    args.remove(0);
                };
            };
            args
        };
        //
        let mut out = Self::default();
        //
        for arg in args {
            out.parse_arg(arg);
        };
        out
    }
}
