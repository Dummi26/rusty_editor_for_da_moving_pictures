use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

pub struct ExternalProgram {
    pub path: PathBuf,
    pub args: Vec<String>,
    mode: ExternalProgramMode,
}
impl ExternalProgram {
    pub fn new(path: PathBuf, mode: ExternalProgramMode) -> Self {
        Self { path, args: vec![], mode, }
    }
    pub fn arg(mut self, arg: String) -> Self { self.args.push(arg); self }
    /// Depending on self.mode, invokes the external program and asks for a new value by providing it with the input. While the input is in bytes, it might be converted to a string using String::from_utf8_lossy first. (This is necessary for program args, but not for stdin). Output is generally collected from stdout.
    pub fn get_next(&self, input: &[u8]) -> Option<Vec<u8>> {
        match self.mode {
            ExternalProgramMode::RunOnceArg => {
                match Command::new(&self.path).args(&self.args).arg(String::from_utf8_lossy(input).as_ref()).stdin(Stdio::null()).stderr(Stdio::null()).output() {
                    Ok(out) => {
                        Some(out.stdout)
                    },
                    Err(_) => None,
                }
            },
            ExternalProgramMode::RunOnceStdin => {
                match Command::new(&self.path).args(&self.args).arg(String::from_utf8_lossy(input).as_ref()).stdin(Stdio::piped()).stderr(Stdio::null()).spawn() {
                    Ok(mut process) => {
                        if let Some(stdin) = &mut process.stdin {
                            if let Ok(_) = stdin.write(input) {
                                if let Ok(_) = stdin.flush() {
                                    if let Ok(out) = process.wait_with_output() {
                                        Some(out.stdout)
                                    } else { None }
                                } else { None }
                            } else { None }
                        } else { None }
                    },
                    Err(_) => None,
                }
            },
        }
    }
}
impl Clone for ExternalProgram {
    fn clone(&self) -> Self {
        let mut o = Self::new(self.path.clone(), self.mode);
        o.args = self.args.clone();
        o
    }
}

#[derive(Clone, Copy)]
pub enum ExternalProgramMode {
    /// Every time a value is needed, the program is launched once. The input is provided as the last argument.
    RunOnceArg,
    /// Every time a value is needed, the program is launched once. The input is provided through stdin, followed by a newline character '\n'.
    RunOnceStdin,
}