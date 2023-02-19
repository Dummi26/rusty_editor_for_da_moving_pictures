use std::io::Write;

pub fn main(mut args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    Cli::new(&mut args).run();
    args
}

struct Cli<'a> {
    // project: crate::project::Project,
    stdout: std::io::Stdout,
    stdin: std::io::Lines<std::io::StdinLock<'a>>,
    project: crate::project::Project,
    project_name: String,
    colored_ouput: bool,
}
impl<'a> Cli<'a> {
    pub fn new(args: &mut crate::cli::CustomArgs) -> Self {
        let stdout = std::io::stdout();
        let mut stdin = std::io::stdin().lines();
        let path = match &args.project_path {
            Some(path) => path,
            None => loop {
                print!("\nNo project path has been specified. Please enter one below or press enter to exit:\n>> ");
                std::io::stdout().flush().unwrap();
                let input = stdin.next().unwrap().unwrap();
                println!("'{}'", input);
            },
        };
        let project = match crate::files::file_handler::read_from_file(path) {
            Err(io_error) => panic!(
                "Could not load '{}'. Are you sure the file exists? (Error: {})",
                path.to_string_lossy().to_string(),
                io_error
            ),
            Ok(Err(parser_error)) => panic!("Could not parse file: {}", parser_error),
            Ok(Ok(project)) => project,
        };
        let project_name = project.proj.lock().unwrap().name.clone();
        Self {
            stdout,
            stdin,
            project,
            project_name,
            colored_ouput: !args.cli_colored_output_disabled,
        }
    }

    fn flush(&mut self) {
        _ = self.stdout.flush();
    }

    fn run(&mut self) {
        loop {
            print!(
                "{}:\n>>",
                self.color(self.project_name.as_str(), ColorStyle::ProjectName)
            );
            self.flush();
            match self.get_line() {
                Ok(line) => match line {
                    CliLine::Exit => break,
                    CliLine::Export(time, w, h, path) => {
                        let vid = self.project.vid();
                        let mut vid = vid.lock().unwrap();
                        if let Some(prep_draw) = vid.prep_draw(time, None) {
                            let mut image = image::DynamicImage::new_rgba8(w, h);
                            let mut settings =
                                crate::video_render_settings::VideoRenderSettings::export(
                                    crate::video_render_settings::FrameRenderInfo::new(
                                        w as f64 / h as f64,
                                    ),
                                );
                            vid.draw(&mut image, prep_draw, &mut settings);
                            if let Err(e) = image.save(&path) {
                                println!(
                                    "{} {}: {}",
                                    self.color("Failed to save image as", ColorStyle::FailError),
                                    self.color(path.as_str(), ColorStyle::FailError),
                                    self.color(format!("{}", e).as_str(), ColorStyle::FailError)
                                );
                            }
                        } else {
                            println!(
                                "{}",
                                self.color("Could not prepare drawing.", ColorStyle::FailError)
                            );
                        }
                    }
                },
                Err((e, fatal)) => {
                    println!(
                        "[unknown] {} {}",
                        self.color("Failed to parse command:", ColorStyle::FailUnknown),
                        self.color(e.as_str(), ColorStyle::FailUnknown)
                    );
                    if fatal {
                        break;
                    }
                }
            }
        }
        println!("Goodbye!");
    }

    pub fn color(&self, txt: &str, color_style: ColorStyle) -> String {
        if self.colored_ouput {
            match color_style {
                ColorStyle::ProjectName => colored::Colorize::bright_green(txt).to_string(), // this shows the beginning/end of commands, so it should be quite distinctive
                ColorStyle::FailUnknown => colored::Colorize::magenta(txt).to_string(),
                ColorStyle::FailError => colored::Colorize::red(txt).to_string(),
            }
        } else {
            txt.to_string()
        }
    }

    fn get_line(&mut self) -> Result<CliLine, (String, bool)> {
        if let Some(Ok(line)) = self.stdin.next() {
            let mut parts = Vec::new();
            // parse by the following rules:
            // '\' in a string segment initiates an escape sequence like \" or \n, which changes the interpretation of the following character
            // '"' starts/ends a string segment
            // ' ' outside of a string segment separates parts of a line
            {
                let mut buf = String::new();
                let mut backslash_escape = false;
                let mut string_mode = false;
                let mut chars = line.chars();
                loop {
                    let ch = chars.next();
                    let mut done = false;
                    if let Some(ch) = ch {
                        if !backslash_escape {
                            match ch {
                                '\\' => {
                                    if string_mode {
                                        backslash_escape = true;
                                    } else {
                                        buf.push(ch);
                                    }
                                }
                                '"' => {
                                    string_mode = !string_mode;
                                }
                                ' ' => {
                                    if !string_mode {
                                        done = true;
                                    }
                                }
                                _ => {
                                    buf.push(ch);
                                }
                            }
                        } else {
                            // '\'-escaped
                            match ch {
                                '\\' => buf.push('\\'),
                                'n' => buf.push('\n'),
                                't' => buf.push('\t'),
                                '"' => buf.push('"'),
                                _ => {
                                    return Err((
                                        format!("Invalid backslash escape sequence '\\{}'!", ch),
                                        false,
                                    ))
                                }
                            }
                        }
                    } else {
                        done = true;
                    }
                    if done {
                        parts.push(std::mem::replace(&mut buf, String::new()));
                        if ch.is_none() {
                            break;
                        };
                    }
                }
            }
            // now parse the actual string
            if let Some(first) = parts.get(0) {
                match first.to_lowercase().as_str() {
                    "exit" => return Ok(CliLine::Exit),
                    "export_frame" | "export frame" => {
                        if let (Some(t), Some(w), Some(h), Some(path)) =
                            (parts.get(1), parts.get(2), parts.get(3), parts.get(4))
                        {
                            if let (Ok(time), Ok(w), Ok(h)) = (t.parse(), w.parse(), h.parse()) {
                                Ok(CliLine::Export(time, w, h, path.to_string()))
                            } else {
                                Err((format!("Could not parse time, width or height of the video. Example of valid input: \"0.0 1920 1080\""), false))
                            }
                        } else {
                            Err((format!("export_frame requires 4 arguments: time, width, height, and output path."), false))
                        }
                    }
                    _ => Err((format!("Command '{}' was not recognized!", first), false)),
                }
            } else {
                Err((format!("Command was empty"), false)) // unreachable, probably...?
            }
        } else {
            Err((format!("no more input. goodbye!"), true))
        }
    }
}

enum CliLine {
    Exit,
    Export(f64, u32, u32, String),
}

enum ColorStyle {
    ProjectName,
    FailUnknown,
    FailError,
}
