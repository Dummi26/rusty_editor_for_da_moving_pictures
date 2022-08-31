use std::{num::ParseFloatError, error::Error, path::PathBuf, io, fmt::{Display, Write}};

pub enum ParserError {
    /// File ended unexpectedly.
    UnexpectedEOF,

    /// Identifier was not 'proj' or 'vid'.
    InvalidIdentifier(String),
    /// Project- or Video info was defined twice.
    DoubleDefinitionOf(String),
    /// Project- or Video info was missing.
    MissingIdentifier(String),

    /// Video type was not List, WithEffect, Raw, ...
    InvalidVideoType(String),

    /// Key was not 'pos', 'start', 'length', ...
    InvalidVideoInfoKey(String),
    /// 'pos', 'start', ... is missing.
    MissingVideoInfoKey(String),

    /// Attempted to load directory full of image frames, but failed to find/read this directory. Maybe missing an external disk and/or permissions?
    DirectoryWithImagesNotFound(PathBuf, io::Error),

    /// This name does not identify an effect.
    UnknownEffect(String),
    /// Effect could not be parsed - custom error provided by the corresponding effect
    EffectParseError{ effect_identifier: String, custom_error: Box<dyn Error>, },

    /// Failed to parse string into a float.
    ParseFloatError(ParseFloatError),
}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ParserError::UnexpectedEOF => format!("Unexpected EOF (end of file)!"),
            ParserError::InvalidIdentifier(i) => format!("Invalid identifier: '{i}' was not 'proj' or 'vid'."),
            ParserError::DoubleDefinitionOf(i) => format!("Identifier '{i}' was defined twice!"),
            ParserError::MissingIdentifier(i) => format!("Identifier '{i}' was never defined, but is required!"),
            ParserError::InvalidVideoType(t) => format!("Video type '{t}' does not exist! Try List, WithEffect, Image, or VidFromImagesInDirectory"),
            ParserError::InvalidVideoInfoKey(k) => format!("VideoInfoKey '{k}' not permitted! Try pos, start, length, or video."),
            ParserError::MissingVideoInfoKey(k) => format!("VideoInfoKey '{k}' was missing but is required! Consider adding it."),
            ParserError::DirectoryWithImagesNotFound(d, e) => format!("Directory with images was not found. Dir: \"{}\", Err: \"{e}\"", d.display()),
            ParserError::UnknownEffect(e) => format!("Effect '{e}' does not exist! Try BlackWhite or Shake."),
            ParserError::EffectParseError { effect_identifier, custom_error } => format!("Failed to parse effect '{effect_identifier}', Err: \"{custom_error}\""),
            ParserError::ParseFloatError(e) => format!("Failed to parse float. Err: {e}"),
        }.as_str())
    }
}