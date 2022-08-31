use std::{str::Chars, path::PathBuf};

use crate::{video::{Video, Pos, VideoType}, project::{Project, ProjectData}, curve::Curve, effect::{effects, Effect}, input_video::InputVideo, multithreading::automatically_cache_frames::VideoWithAutoCache};

use super::parser_general::ParserError;

pub fn parse(str: &str, path: PathBuf) -> Result<Project, ParserError> {
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
            "vid" => match vid { None => vid = Some(Video::new_full(VideoType::List(parse_vid_vids(&mut chars)?))), Some(_) => return Err(ParserError::DoubleDefinitionOf(identifier)), },
            _ => return Err(ParserError::InvalidIdentifier(identifier)),
        };
    };
    // return
    match (proj, vid) {
        (Some(proj), Some(vid)) => Ok(Project { proj, vid: VideoWithAutoCache::new(vid, 0, 0), }),
        (None, _) => Err(ParserError::MissingIdentifier(format!("proj"))),
        (_, None) => Err(ParserError::MissingIdentifier(format!("vid"))),
    }
}

fn parse_proj(chars: &mut Chars, path: PathBuf) -> Result<ProjectData, ParserError> {
    Ok(ProjectData {
        name: format!("doesn't_matter"),
        path,
    })
}

fn parse_vid(chars: &mut Chars) -> Result<Video, ParserError> {
    let mut pos = None;
    let mut start = None;
    let mut length = None;
    let mut video = None;
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
            "pos" => pos = Some(Pos { x: parse_vid_curve(chars)?, y: parse_vid_curve(chars)?, w: parse_vid_curve(chars)?, h: parse_vid_curve(chars)?, }),
            "start" => start = Some(parse_vid_f64(chars)?),
            "length" => length = Some(parse_vid_f64(chars)?),
            "video" => video = Some(parse_vid_video(chars)?),
            _ => return Err(ParserError::InvalidVideoInfoKey(identifier)),
        };
    };
    match (pos, start, length, video) {
        (Some(pos), Some(start_frame), Some(length), Some(video)) => Ok(Video::new(pos, start_frame, length, video)),
        (None, _, _, _) => Err(ParserError::MissingVideoInfoKey(format!("pos"))),
        (_, None, _, _) => Err(ParserError::MissingVideoInfoKey(format!("start"))),
        (_, _, None, _) => Err(ParserError::MissingVideoInfoKey(format!("length"))),
        (_, _, _, None) => Err(ParserError::MissingVideoInfoKey(format!("video"))),
    }
}

fn parse_vid_vids(chars: &mut Chars) -> Result<Vec<Video>, ParserError> {
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

fn parse_vid_video(chars: &mut Chars) -> Result<VideoType, ParserError> {
    let mut identifier = String::new();
    loop {
        match chars.next() {
            Some(':') => break,
            Some(ch) => identifier.push(ch),
            None => return Err(ParserError::UnexpectedEOF),
        };
    };
    return Ok(match identifier.as_str() {
        "List" => {
            VideoType::List(parse_vid_vids(chars)?)
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
            VideoType::WithEffect(Box::new(video_data), match effect_name.as_str() {
                "BlackWhite" => Effect::new(effects::BlackWhite {}),
                "Shake" => Effect::new(effects::Shake {shake_dist_x: parse_vid_f64(chars)?, shake_dist_y: parse_vid_f64(chars)?, shakes_count_x: parse_vid_f64(chars)?, shakes_count_y: parse_vid_f64(chars)?, }),
                _ => return Err(ParserError::UnknownEffect(effect_name)),
            })
        },
        "Image" => {
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
            VideoType::Image(crate::content::image::Image::new(path))
        }
        "VidFromImagesInDirectory" => {
            let mut directory = std::path::PathBuf::from("/");
            let mut directory_current = String::new();
            loop {
                match chars.next() {
                    Some('/') => {
                        directory.push(directory_current);
                        directory_current = String::new();
                    },
                    Some('\\') => {
                        if directory_current.len() != 0 { directory.push(directory_current); };
                        break;
                    },
                    Some(ch) => directory_current.push(ch),
                    None => return Err(ParserError::UnexpectedEOF),
                };
            };
            VideoType::Raw(
                match InputVideo::new_from_directory_full_of_frames(directory.clone()) {
                    Ok(v) => v,
                    Err(err) => return Err(ParserError::DirectoryWithImagesNotFound(directory, err)),
                }
            )
        },
        _ => return Err(ParserError::InvalidVideoType(identifier)),
    })
}

fn parse_vid_curve(chars: &mut Chars) -> Result<Curve, ParserError> {
    Ok(match chars.next() {
        Some('=') => Curve::Constant(parse_vid_f64(chars)?),
        Some('/') => Curve::Linear(parse_vid_f64(chars)?, parse_vid_f64(chars)?),
        Some('#') => Curve::Chain(
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
        _ => return Err(ParserError::UnexpectedEOF),
    })
}

fn parse_vid_f64(chars: &mut Chars) -> Result<f64, ParserError> {
    let mut str = String::new();
    loop {
        match chars.next() {
            Some(ch) => match ch {
                ';' => return match str.parse() { Ok(v) => Ok(v), Err(err) => Err(ParserError::ParseFloatError(err)), },
                _ => str.push(ch),
            },
            None => return Err(ParserError::UnexpectedEOF),
        };
    };
}