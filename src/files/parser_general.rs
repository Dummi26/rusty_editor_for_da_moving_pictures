use std::{num::{ParseFloatError, ParseIntError}, path::PathBuf, io, fmt::Display};

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

    /// Tried to parse a curve, but the first char was not one of the allowed alignment chars
    InvalidPosAlignment(char),

    /// Attempted to load directory full of image frames, but failed to find/read this directory. Maybe missing an external disk and/or permissions?
    DirectoryWithImagesNotFound(PathBuf, io::Error),
    
    /// Only =, *, and L are allowed. For None, do not include this option in the save file at all (None is the default value).
    InvalidTransparencyAdjustmentIdentifier(char),
    
    /// Attempted to parse a curve, but found an unexpected character.
    InvalidCurveIdentifier(char),

    /// Invalid type for a text
    InvalidTextType(char),
    VideoFileFailedToParseStartOrEndFrame(String),

    /// This name does not identify an effect.
    UnknownEffect(String),
    /// Effect could not be parsed - custom error for the corresponding effect.
    EffectParseError{ effect_identifier: String, custom_error: String, },

    /// Failed to parse string into an int.
    ParseIntError(String, ParseIntError),
    /// Failed to parse string into a float.
    ParseFloatError(String, ParseFloatError),

    /// When no other error type matches
    Todo
}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnexpectedEOF => format!("Unexpected EOF (end of file)!"),
            Self::InvalidIdentifier(i) => format!("Invalid identifier: '{i}' was not 'proj' or 'vid'."),
            Self::DoubleDefinitionOf(i) => format!("Identifier '{i}' was defined twice!"),
            Self::MissingIdentifier(i) => format!("Identifier '{i}' was never defined, but is required!"),
            Self::InvalidVideoType(t) => format!("Video type '{t}' does not exist! Try List, WithEffect, Image, or VidFromImagesInDirectory"),
            Self::InvalidVideoInfoKey(k) => format!("VideoInfoKey '{k}' not permitted! Try pos, start, length, video or transparency_adjustments."),
            Self::MissingVideoInfoKey(k) => format!("VideoInfoKey '{k}' was missing but is required! Consider adding it."),
            Self::DirectoryWithImagesNotFound(d, e) => format!("Directory with images was not found. Dir: \"{}\", Err: \"{e}\"", d.display()),
            Self::InvalidTransparencyAdjustmentIdentifier(i) => format!("Invalid transparency adjustment identifier '{i}'. Only = (force), * (factor), and L (fully opaque unless fully transparent) are allowed. (= and * must be followed by Curves.)"),
            Self::InvalidCurveIdentifier(c) => format!("Found unexpected character '{c}' when parsing Curve. Allowed are only 0-9, '.', '/', 's', and '#'."),
            Self::InvalidTextType(c) => format!("Found unexpected text type character '{c}'. Use 's' for static text."),
            Self::VideoFileFailedToParseStartOrEndFrame(t) => format!("Failed to parse a video's start and end frames (crop): {t}"),
            Self::UnknownEffect(e) => format!("Effect '{e}' does not exist! Try None (placeholder), BlackWhite, Shake, ChangeSpeed, Blur, ColorAdjust or ColorKey."),
            Self::EffectParseError { effect_identifier, custom_error } => format!("Failed to parse effect '{effect_identifier}', Err: \"{custom_error}\""),
            Self::ParseIntError(i, e) => format!("Failed to parse '{i}' into an int. Err: {e}"),
            Self::ParseFloatError(i, e) => format!("Failed to parse '{i}' into a float. Err: {e}"),
            Self::InvalidPosAlignment(c) => format!("Failed to get alignment of position: First char after 'pos:' was {c}, but only 1^2<+>3v4 are allowed."),
            Self::Todo => format!("There was an error, but no error message has been implemented yet."),
        }.as_str())
    }
}