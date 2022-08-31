use std::{num::{ParseFloatError, ParseIntError}, error::Error, path::PathBuf, io, fmt::{Display, Write}};

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
    
    /// Only =, *, and L are allowed. For None, do not include this option in the save file at all (None is the default value).
    InvalidTransparencyAdjustmentIdentifier(char),
    
    /// Attempted to parse a curve, but found an unexpected character.
    InvalidCurveIdentifier(char),

    /// This name does not identify an effect.
    UnknownEffect(String),
    /// Effect could not be parsed - custom error for the corresponding effect.
    EffectParseError{ effect_identifier: String, custom_error: String, },

    /// Failed to parse string into an int.
    ParseIntError(ParseIntError),
    /// Failed to parse string into a float.
    ParseFloatError(ParseFloatError),
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
            Self::UnknownEffect(e) => format!("Effect '{e}' does not exist! Try None (placeholder), BlackWhite, Shake, ChangeSpeed, Blur, ColorAdjust or ColorKey."),
            Self::EffectParseError { effect_identifier, custom_error } => format!("Failed to parse effect '{effect_identifier}', Err: \"{custom_error}\""),
            Self::ParseIntError(e) => format!("Failed to parse int. Err: {e}"),
            Self::ParseFloatError(e) => format!("Failed to parse float. Err: {e}"),
        }.as_str())
    }
}