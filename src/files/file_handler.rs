use std::{error::Error, fmt::{Display, Debug}, io, path::PathBuf};

use crate::{project::Project};

const VERSION_CURRENT_MAJOR: u32 = 0;
const VERSION_CURRENT_MINOR: u32 = 0;

pub fn read_from_file(file_name: &PathBuf) -> io::Result<Result<Project, CreateVideoFromFileError>> {
    let file_contents = std::fs::read_to_string(&file_name)?;
    let mut file_content_lines = file_contents.lines();
    if let Some(first_line) = file_content_lines.next() {
        let first_line_split = Vec::from_iter(first_line.splitn(2, '.'));
        if first_line_split.len() < 2 { return Ok(Err(CreateVideoFromFileError::CouldNotDecodeVersion)); };
        let version_major = match first_line_split[0].parse::<u32>() { Ok(v) => v, Err(err) => return Ok(Err(CreateVideoFromFileError::CouldNotDecodeMajorVersion(err))),};
        let version_minor = match first_line_split[1].parse::<u32>() { Ok(v) => v, Err(err) => return Ok(Err(CreateVideoFromFileError::CouldNotDecodeMinorVersion(err))),};
        if version_major > VERSION_CURRENT_MAJOR { return Ok(Err(CreateVideoFromFileError::VersionTooNewMajor)); };
        if version_major == VERSION_CURRENT_MAJOR && version_minor > VERSION_CURRENT_MINOR { return Ok(Err(CreateVideoFromFileError::VersionTooNewMinor)); };
        //
        let mut file_content_final = String::new();
        for line in file_content_lines {
            let mut state = 0;
            for char in line.chars() {
                match state {
                    0 => state0(char, &mut state, &mut file_content_final),
                    1 => state1(char, &mut state, &mut file_content_final),
                    _ => break,
                };
                fn state0(char: char, state: &mut u32, file_content_final: &mut String) {
                    match char {
                        // ignore spaces and tabs at the start of a line
                        ' ' | '\t' => (),
                        // any other character switches state to 1 and calls state1() as this char is already part of the line's actual contents.
                        _ => {
                            *state = 1;
                            state1(char, state, file_content_final);
                        },
                    };
                }
                fn state1(char: char, _state: &mut u32, file_content_final: &mut String) {
                    file_content_final.push(char);
                }
            };
        };
        //
        if version_major == VERSION_CURRENT_MAJOR {
            match super::parser_v0::parse(file_content_final.as_str(), file_name) {
                Ok(v) => Ok(Ok(v)),
                Err(err) => Ok(Err(CreateVideoFromFileError::ParseError { file_version: (version_major, version_minor), parser_version: (VERSION_CURRENT_MAJOR, VERSION_CURRENT_MINOR), parser_error: err, })),
            }
        } else {
            Ok(Err(CreateVideoFromFileError::NoParserForVersion((version_major, version_minor))))
        }
    } else {
        Ok(Err(CreateVideoFromFileError::NoFirstLine))
    }
}

pub enum CreateVideoFromFileError {
    NoFirstLine,
    /// The first line did not contain the . required by the major.minor version format
    CouldNotDecodeVersion,
    /// The first line matched the major.minor format, but major could not be parsed to an int
    CouldNotDecodeMajorVersion(std::num::ParseIntError),
    /// The first line matched the major.minor format, but minor could not be parsed to an int
    CouldNotDecodeMinorVersion(std::num::ParseIntError),
    /// The file's major version was newer than the newest major version this version of the program can read.
    VersionTooNewMajor,
    /// The file's minor version was newer than the newest minor version this version of the program can read. (This requires that the Major version matched)
    VersionTooNewMinor,
    /// There is no parser for the given file version.
    NoParserForVersion((u32, u32)),
    /// There was an error parsing the file.
    ParseError { file_version: (u32, u32), parser_version: (u32, u32), parser_error: super::parser_general::ParserError, },
}

impl Error for CreateVideoFromFileError {}
impl Display for CreateVideoFromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("CreateVideoFromFileError: {}",
            match self {
                CreateVideoFromFileError::NoFirstLine => format!("The specified file did not contain a first line and is therefor useless. This might also indicate file permission issues if the file contains something."),
                CreateVideoFromFileError::CouldNotDecodeVersion => format!("Could not decode version. Ensure the first line of the project file contains a valid version in the major.minor format, where major and minor are positive integers."),
                CreateVideoFromFileError::CouldNotDecodeMajorVersion(e) => format!("Could not decode major version: {e}"),
                CreateVideoFromFileError::CouldNotDecodeMinorVersion(e) => format!("Could not decode minor version: {e}"),
                CreateVideoFromFileError::VersionTooNewMajor => format!("The major version of this file exceeds the major version of the parser. Please update the program. Manually changing the version in the file WILL NOT WORK, as a major version change indicates significant syntax changes."),
                CreateVideoFromFileError::VersionTooNewMinor => format!("The minor version of this file exceeds the minor version of the parser. Please update the program or manually change the version in the file (this might cause further parsing errors, but it might also work if no new features are used - it's only a minor version after all.)"),
                CreateVideoFromFileError::NoParserForVersion((maj, min)) => format!("There is no parser for version {maj}.{min}. (Upgrading from old file versions is not supported yet, sorry.)"),
                CreateVideoFromFileError::ParseError { file_version, parser_version, parser_error } => format!("ParserError [{}.{} in {}.{}]: ({})", file_version.0, file_version.1, parser_version.0, parser_version.1, parser_error),
            }
        ).as_str())
    }
}
impl Debug for CreateVideoFromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}