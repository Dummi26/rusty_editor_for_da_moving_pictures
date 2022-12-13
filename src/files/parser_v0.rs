use std::{str::{Chars, FromStr}, path::PathBuf};

use crate::{video::{Video, Pos, VideoType, VideoTypeEnum, CompositingMethod}, project::{Project, ProjectData}, curve::Curve, effect::{effects, Effect}, content::input_video::InputVideo};

use super::parser_general::ParserError;

pub fn parse(str: &str, path: &PathBuf) -> Result<Project, ParserError> {
    let mut chars = str.chars();
    
    let mut proj = None;
    let mut vid = None;
    
    'before_return: loop {
        // find out what we are parsing
        let mut identifier = String::new();
        loop {
            let char = if let Some(ch) = chars.next() { ch } else { break 'before_return; };
            if char == ':' { break; };
            identifier.push(char);
        };
        match identifier.as_str() {
            "proj" => match proj { None => proj = Some(parse_proj(&mut chars, path.clone())?), Some(_) => return Err(ParserError::DoubleDefinitionOf(identifier)), },
            "vid" => match vid { None => vid = Some(parse_vid(&mut chars)?), Some(_) => return Err(ParserError::DoubleDefinitionOf(identifier)), },
            _ => return Err(ParserError::InvalidIdentifier(identifier)),
        };
    };
    // return
    match (proj, vid) {
        (Some(proj), Some(vid)) => Ok(Project { proj: std::sync::Arc::new(std::sync::Mutex::new(proj)), vid: std::sync::Arc::new(std::sync::Mutex::new(vid)), }),
        (None, _) => Err(ParserError::MissingIdentifier(format!("proj"))),
        (_, None) => Err(ParserError::MissingIdentifier(format!("vid"))),
    }
}

pub fn parse_proj(chars: &mut Chars, path: PathBuf) -> Result<ProjectData, ParserError> {
    Ok(ProjectData {
        name: format!("doesn't_matter"),
        path: Some(path),
        render_settings_export: Some(crate::video_render_settings::VideoRenderSettings::export(crate::video_render_settings::FrameRenderInfo::new(1.0))), // TODO: This is a default value - project metadata should also be saved in the project file!
    })
}

pub fn parse_vid(chars: &mut Chars) -> Result<Video, ParserError> {
    let mut pos = None;
    let mut start = None;
    let mut length = None;
    let mut video = None;
    let mut compositing = None;
    'before_return: loop {
        let mut identifier = String::new();
        loop {
            let char = match chars.next() { Some(ch) => ch, None => return Err(ParserError::UnexpectedEOF), };
            match char {
                ':' => break,
                _ => identifier.push(char),
            };
        };
        match identifier.as_str() {
            "" => break 'before_return,
            "pos" => pos = Some(Pos { align: match chars.next() { None => return Err(ParserError::UnexpectedEOF),
                Some('^') => crate::video::PosAlign::Top,
                Some('v') => crate::video::PosAlign::Bottom,
                Some('<') => crate::video::PosAlign::Left,
                Some('>') => crate::video::PosAlign::Right,
                Some('+') => crate::video::PosAlign::Center,
                Some('1') => crate::video::PosAlign::TopLeft,
                Some('2') => crate::video::PosAlign::TopRight,
                Some('3') => crate::video::PosAlign::BottomLeft,
                Some('4') => crate::video::PosAlign::BottomRight,
                Some('!') => crate::video::PosAlign::Custom(parse_vid_curve(chars)?, parse_vid_curve(chars)?),
                Some(c) => return Err(ParserError::InvalidPosAlignment(c)),
            }, x: parse_vid_curve(chars)?, y: parse_vid_curve(chars)?, w: parse_vid_curve(chars)?, h: parse_vid_curve(chars)?, }),
            "start" => start = Some(parse_vid_f64(chars)?),
            "length" => length = Some(parse_vid_f64(chars)?),
            "video" => video = Some(parse_vid_video(chars)?),
            "compositing" => compositing = Some(match chars.next() {
                Some('_') => CompositingMethod::Ignore,
                Some('=') => CompositingMethod::Opaque,
                Some('|') => CompositingMethod::Direct, // "Pipe"
                Some('*') => CompositingMethod::TransparencySupport,
                Some(ch) => return Err(ParserError::InvalidCompositingMode(ch)),
                None => return Err(ParserError::UnexpectedEOF),
            }),
            _ => return Err(ParserError::InvalidVideoInfoKey(identifier)),
        };
    };
    match (
        match pos { None => { Pos { x: Curve::Constant(0.), y: Curve::Constant(0.), w: Curve::Constant(1.), h: Curve::Constant(1.), align: crate::video::PosAlign::TopLeft } }, Some(v) => v },
        match start { None => 0.0, Some(v) => v },
        match length { None => 1.0, Some(v) => v },
        video
    ) {
        (/*Some(*/pos, /*Some(*/start, /*Some(*/length, Some(video)) => Ok({
            let mut vid = Video::new(pos, start, length, video);
            vid.compositing = compositing;
            vid
        }),
        // (None, _, _, _) => Err(ParserError::MissingVideoInfoKey(format!("pos"))),
        // (_, None, _, _) => Err(ParserError::MissingVideoInfoKey(format!("start"))),
        // (_, _, None, _) => Err(ParserError::MissingVideoInfoKey(format!("length"))),
        (_, _, _, None) => Err(ParserError::MissingVideoInfoKey(format!("video"))),
    }
}

pub fn parse_vid_vids(chars: &mut Chars) -> Result<Vec<Video>, ParserError> {
    let mut vec = Vec::new();
    loop {
        match chars.next() {
            Some('+') => vec.push(parse_vid(chars)?),
            Some(_ /* preferrably ; for clarity. */) => break,
            None => return Err(ParserError::UnexpectedEOF),
        };
    };
    Ok(vec)
}


pub fn parse_vid_video(chars: &mut Chars) -> Result<VideoType, ParserError> {
    let identifier = {
        let mut i = String::new();
        loop {
            match chars.next() {
                Some(':') => break,
                Some(' ' | '\t') => continue,
                Some(ch) => i.push(ch),
                None => return Err(ParserError::UnexpectedEOF),
            };
        }
        i
    };
    return Ok(VideoType::new(match identifier.as_str() {
        "List" => {
            VideoTypeEnum::List(parse_vid_vids(chars)?)
        },
        "AspectRatio" => {
            let (w, h) = (parse_vid_curve(chars)?, parse_vid_curve(chars)?);
            VideoTypeEnum::AspectRatio(Box::new(parse_vid(chars)?), w, h)
        },
        "WithEffect" => {
            let video_data = parse_vid(chars)?;
            let effect_name = {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some(':') => break name,
                        Some(ch) => name.push(ch),
                        None => return Err(ParserError::UnexpectedEOF),
                    };
                }
            };
            VideoTypeEnum::WithEffect(Box::new(video_data), match effect_name.as_str() {
                "None" => Effect::new(effects::Nothing {}),
                "BlackWhite" => Effect::new(effects::BlackWhite {}),
                "Shake" => Effect::new(effects::Shake {shake_dist_x: parse_vid_f64(chars)?, shake_dist_y: parse_vid_f64(chars)?, shakes_count_x: parse_vid_f64(chars)?, shakes_count_y: parse_vid_f64(chars)?, }),
                "ChangeTime" => Effect::new(effects::ChangeTime { time: parse_vid_curve(chars)?, }),
                "Blur" => Effect::new(effects::Blur { mode: {
                    let mut identifier = String::new();
                    loop {
                        match chars.next() {
                            Some(':') => break,
                            Some(ch) => identifier.push(ch),
                            None => return Err(ParserError::UnexpectedEOF),
                        };
                    };
                    match identifier.as_str() {
                        "Square" => effects::Blur_Mode::Square { radius: parse_vid_curve(chars)?, },
                        "Downscale" => effects::Blur_Mode::Downscale { width: parse_vid_curve(chars)?, height: parse_vid_curve(chars)?, },
                        _ => return Err(ParserError::EffectParseError { effect_identifier: effect_name, custom_error: format!("Blur mode '{identifier}' does not exist! Try Square (Curve) or Downscale (Curve + Curve)"), }),
                    }
                }, }),
                "ColorAdjust" => Effect::new(effects::ColorAdjust { mode: {
                    let mut identifier = String::new();
                    loop {
                        match chars.next() {
                            Some(':') => break,
                            Some(ch) => identifier.push(ch),
                            None => return Err(ParserError::UnexpectedEOF),
                        };
                    };
                    match identifier.as_str() {
                        "rgba" => effects::ColorAdjust_Mode::Rgba(parse_vid_curve(chars)?, parse_vid_curve(chars)?, parse_vid_curve(chars)?, parse_vid_curve(chars)?),
                        _ => return Err(ParserError::EffectParseError { effect_identifier: effect_name, custom_error: format!("'{}' is not a valid ColorAdjustMode. Try rgba:RGBA where R,G,B,A are Curve.", identifier), })
                    }
                }, }),
                "ColorKey" => Effect::new(effects::ColorKey { mode: {
                    let mut identifier = String::new();
                    loop {
                        match chars.next() {
                            Some(':') => break,
                            Some(ch) => identifier.push(ch),
                            None => return Err(ParserError::UnexpectedEOF),
                        };
                    };
                    match identifier.as_str() {
                        "rgb_eq" => effects::ColorKey_Mode::TransparentIfMatches((parse_vid_int(chars)?, parse_vid_int(chars)?, parse_vid_int(chars)?)),
                        "rgb_rng" => effects::ColorKey_Mode::TransparentIfRange(((parse_vid_int(chars)?, parse_vid_int(chars)?), (parse_vid_int(chars)?, parse_vid_int(chars)?), (parse_vid_int(chars)?, parse_vid_int(chars)?))),
                        _ => return Err(ParserError::EffectParseError { effect_identifier: effect_name, custom_error: format!("'{}' is not a valid ColorKeyMode. Try rgb_eq:R;G;B where R,G,B are int-u8, or rgb_rng:R1;R2;G1;G2;B1;B2 where R1,R2,G1,G2,B1,B2 are int-u8.", identifier), })
                    }
                }, }),
                _ => return Err(ParserError::UnknownEffect(effect_name)),
            })
        },
        "Text" => {
            let font_path = parse_path(chars)?;
            let font_index = parse_vid_int(chars)?;
            let color = crate::types::Color::parse(chars)?;
            let mut text = crate::content::text::Text::new(
                match chars.next() {
                    Some('s') => crate::content::text::TextType::Static(parse_string(chars)?),
                    Some('!') => {
                        match chars.next() {
                            Some(_) => { // TODO: add more modes and options for args and stuff, also add an error for wrong char here
                                let path = parse_path(chars)?;
                                crate::content::text::TextType::Program(crate::external_program::ExternalProgram::new(path, crate::external_program::ExternalProgramMode::RunOnceArg))
                            },
                            None => return Err(ParserError::UnexpectedEOF),
                        }
                    },
                    Some(c) => return Err(ParserError::InvalidTextType(c)),
                    None => return Err(ParserError::UnexpectedEOF),
                }
            );
            text.set_color(color);
            if let Ok(file) = std::fs::read(&font_path) {
                if let Some(font) = rusttype::Font::try_from_vec_and_index(file, font_index) {
                    text.set_font(font.into());
                } else { println!("Font '{}' could not be parsed (using the ttf_parser crate)", font_path.to_string_lossy().as_ref()); }
            } else { println!("Font file '{}' does not exist!", font_path.to_string_lossy().as_ref()); };
            VideoTypeEnum::Text(text)
        },
        "Image" => {
            VideoTypeEnum::Image(crate::content::image::Image::new(parse_path(chars)?))
        }
        "VidFromImagesInDirectory" => {
            let directory = parse_path(chars)?;
            let crop = {
                let mut first = String::new();
                let mut second = String::new();
                let mut rev = None;
                loop {
                    match chars.next() {
                        Some('-') => if rev.is_none() { rev = Some(false); },
                        Some('+') => if rev.is_none() { rev = Some(true); },
                        Some(';') => break,
                        Some(c) => match rev {
                            None => first.push(c),
                            Some(_) => second.push(c),
                        },
                        None => return Err(ParserError::UnexpectedEOF),
                    }
                };
                if let Some(rev) = rev {
                    // TODO: better errors (not just parse int error)?
                    let frame1: u32 = match first.parse() { Ok(v) => v, Err(e) => return Err(ParserError::ParseIntError(first, e)) };
                    let frame2: u32 = match second.parse() { Ok(v) => v, Err(e) => return Err(ParserError::ParseIntError(second, e)) };
                    (frame1, frame2, rev)
                } else {
                    return Err(ParserError::VideoFileFailedToParseStartOrEndFrame("the two numbers were not separated by a + or - symbol. (';' too early)".to_string()));
                }
            };
            VideoTypeEnum::Raw(
                match InputVideo::new_from_directory_full_of_frames(directory.clone(), crop) {
                    Ok(v) => v,
                    Err(err) => return Err(ParserError::DirectoryWithImagesNotFound(directory, err)),
                }
            )
        },
        "VidUsingFfmpeg" => {
            VideoTypeEnum::Ffmpeg(
                crate::content::ffmpeg_vid::FfmpegVid::new(parse_path(chars)?)
            )
        },
        _ => return Err(ParserError::InvalidVideoType(identifier)),
    }))
}

pub fn parse_vid_curve(chars: &mut Chars) -> Result<Curve, ParserError> {
    Ok(loop { break match chars.next() {
        Some(char) => match char {
            ' ' | '\t' => continue,
            '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.' => Curve::Constant(parse_vid_f64_prepend(String::from(char), chars)?),
            '/' => Curve::Linear(parse_vid_curve(chars)?.b(), parse_vid_curve(chars)?.b()),
            's' => Curve::SmoothFlat(parse_vid_curve(chars)?.b(), parse_vid_curve(chars)?.b()),
            '#' => Curve::Chain(
                {
                    let mut vec = Vec::new();
                    loop {
                        match chars.next() {
                            Some('+') => {
                                let f = parse_vid_f64(chars)?;
                                vec.push((parse_vid_curve(chars)?, f));
                            },
                            None => return Err(ParserError::UnexpectedEOF),
                            Some(_ /* preferrably # for clarity, but can be any character except +. */) => break,
                        };
                    };
                    vec
                }
            ),
            '!' => Curve::Program(crate::external_program::ExternalProgram::new(parse_path(chars)?, crate::external_program::ExternalProgramMode::RunOnceArg), crate::curve::CurveExternalProgramMode::String), // TODO: Make this more flexible
            _ => return Err(ParserError::InvalidCurveIdentifier(char)),
        },
        None => return Err(ParserError::UnexpectedEOF),
    }; })
}

/// Parses an integer in the form "(int);"
pub fn parse_vid_int<T>(chars: &mut Chars) -> Result<T, ParserError> where T: FromStr<Err = std::num::ParseIntError> {
    let str = parse_vid_to_next_semicolon_errors(String::new(), chars)?;
    match str.parse() {
        Ok(v) => Ok(v),
        Err(err) => Err(ParserError::ParseIntError(str, err)),
    }
}

pub fn parse_vid_f64_prepend(prepend: String, chars: &mut Chars) -> Result<f64, ParserError> {
    let str = parse_vid_to_next_semicolon_errors(prepend, chars)?;
    match str.parse() {
        Ok(v) => Ok(v),
        Err(err) => Err(ParserError::ParseFloatError(str, err)),
    }
}
pub fn parse_vid_f64(chars: &mut Chars) -> Result<f64, ParserError> {
    parse_vid_f64_prepend(String::new(), chars)
}

/// Same as the one without _errors, but (Ok(s), _) becomes Ok(s) while (Err(s), _) becomes Err(ParserError::UnexpectedEOF)
pub fn parse_vid_to_next_semicolon_errors(prepend: String, chars: &mut Chars) -> Result<String, ParserError> {
    if let Ok(text) = parse_vid_to_next_semicolon(prepend, chars).0 {
        Ok(text)
    } else {
        Err(ParserError::UnexpectedEOF)
    }
}

/// The first tuple member can be Ok(prepend+chars) where chars are all chars until the first semicolon
///                           or Err(prepend+chars) where chars are all chars until the iterator returned None.
/// The second tuple member can be discarded, it contains mostly debugging information. See the fn definition for more info.
pub fn parse_vid_to_next_semicolon(mut prepend: String, chars: &mut Chars) -> (Result<String, String>, (u32, u32)) {
    let mut chars_added = 0;
    let mut chars_discarded = 0;
    loop {
        match chars.next() {
            Some(';') => break,
            Some(' ' | '\t') => chars_discarded += 1,
            Some(ch) => {
                prepend.push(ch);
                chars_added += 1;
            },
            None => return (Err(prepend), (chars_added, chars_discarded)),
        };
    };
    return (Ok(prepend), (chars_added, chars_discarded))
}

/// Reads all chars into a buffer, stopping at '\!', and interpreting '\\' as '\', '\n' as newline, etc. \x with an unknown x will be interpreted litterally, but this is unreliable, so please remember to replace all '\'s with '\\' when saving!
pub fn parse_string(chars: &mut Chars) -> Result<String, ParserError> {
    let mut buf = String::new();
    let mut backslash = false;
    loop {
        if let Some(ch) = chars.next() {
            if ! backslash {
                match ch {
                    '\\' => backslash = true,
                    c => buf.push(c),
                }
            } else {
                match ch {
                    'n' => buf.push('\n'),
                    '\\' => buf.push('\\'),
                    't' => buf.push('\t'),
                    '!' => break Ok(buf), // the sequence \! terminates the string
                    c => { buf.push('\\'); buf.push(c); },
                }
            }
        } else { return Err(ParserError::UnexpectedEOF); }
    }
}

pub fn parse_path(chars: &mut Chars) -> Result<std::path::PathBuf, ParserError> {
    let mut path = std::path::PathBuf::from("/");
    let mut path_current = String::new();
    loop {
        match chars.next() {
            Some('/') => {
                path.push(path_current);
                path_current = String::new();
            },
            Some('\\') => {
                if path_current.len() != 0 { path.push(path_current); };
                break;
            },
            Some(ch) => path_current.push(ch),
            None => return Err(ParserError::UnexpectedEOF),
        };
    };
    Ok(path)
}