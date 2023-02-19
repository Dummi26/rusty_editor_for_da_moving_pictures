use std::{
    default::Default,
    io::{self, Read},
    path::PathBuf,
};

use image::DynamicImage;

use crate::project::Project;

use super::content::{Content, GenericContentData};

pub struct Text {
    text: TextType,
    font: Option<rusttype::Font<'static>>,
    color: crate::types::Color,
    generic_content_data: crate::content::content::GenericContentData,
    pub as_content_changes: TextChanges,
}

#[derive(Default)]
pub struct TextChanges {
    pub text: Option<TextType>,
    pub font: Option<rusttype::Font<'static>>,
    pub color: Option<crate::types::Color>,
}

#[derive(Clone)]
pub enum TextType {
    Static(String),
    Program(crate::external_program::ExternalProgram),
}

impl Content for Text {
    fn clone_no_caching(&self) -> Self {
        Self::new(self.text.clone(), self.generic_content_data.reset())
    }

    fn children(&self) -> Vec<&Self> {
        Vec::new()
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        Vec::new()
    }

    fn has_changes(&self) -> bool {
        self.as_content_changes.text.is_some()
            | self.as_content_changes.font.is_some()
            | self.as_content_changes.color.is_some()
    }
    fn apply_changes(&mut self) -> bool {
        let mut o = false;
        if let Some(text) = self.as_content_changes.text.take() {
            self.set_text(text);
            o = true;
        }
        if let Some(font) = self.as_content_changes.font.take() {
            self.font = Some(font);
            o = true;
        }
        if let Some(color) = self.as_content_changes.color.take() {
            self.color = color;
            o = true;
        }
        o
    }

    fn generic_content_data(&mut self) -> &mut super::content::GenericContentData {
        &mut self.generic_content_data
    }
}
impl Text {
    pub fn new(text: TextType, generic_content_data: GenericContentData) -> Self {
        Self {
            text,
            font: None,
            color: crate::types::Color::RGBA(
                crate::curve::CurveData::Constant(1.0).into(),
                crate::curve::CurveData::Constant(1.0).into(),
                crate::curve::CurveData::Constant(1.0).into(),
                crate::curve::CurveData::Constant(1.0).into(),
            ),
            as_content_changes: TextChanges::default(),
            generic_content_data,
        }
    }
}
impl Text {
    pub fn set_text(&mut self, new: TextType) {
        self.text = new;
    }
    pub fn set_font(&mut self, new: rusttype::Font<'static>) {
        self.font = Some(new);
    }
    pub fn set_color(&mut self, new: crate::types::Color) {
        self.color = new;
    }
    pub fn text(&self) -> &TextType {
        &self.text
    }
    pub fn get_text(&self, prog: f64) -> String {
        match &self.text {
            TextType::Static(text) => text.to_string(),
            TextType::Program(p) => match p.get_next(format!("{}", prog).as_bytes()) {
                Some(out) => match String::from_utf8(out) {
                    Ok(v) => v.trim_end().to_string(),
                    Err(_) => "external program: output wasn't valid UTF-8!".to_string(),
                },
                None => "external program: couldn't launch or couldn't get output".to_string(),
            },
        }
    }
    pub fn draw(
        &mut self,
        image: &mut DynamicImage,
        prep_draw: &crate::video::PrepDrawData,
        align_anchor: (f64, f64),
    ) {
        let position = prep_draw.pos_px;
        let text = self.get_text(prep_draw.progress);
        if let Some(font) = &self.font {
            let c = self.color.get_rgba(prep_draw.progress); // TODO: replace image.height()/width() with position.2/3!
            let dimensions = imageproc::drawing::text_size(
                rusttype::Scale::uniform(position.3 as f32),
                font,
                &text,
            );
            let (factor, offset_x, offset_y) = if dimensions.0 as f64 > position.2 {
                let f = position.2 / dimensions.0 as f64;
                (
                    f,
                    0,
                    ((position.3 - dimensions.1 as f64 * f) * align_anchor.1).round() as _,
                )
            } else {
                (
                    1.0,
                    ((position.2 - dimensions.0 as f64/* free space */) * align_anchor.0).round()
                        as _,
                    0,
                )
            };
            let height = (position.3 * factor) as _;
            match &prep_draw.compositing {
                crate::video::CompositingMethod::Ignore => (),
                crate::video::CompositingMethod::Opaque => {
                    imageproc::drawing::draw_text_mut(
                        image,
                        image::Rgba {
                            0: [
                                (255.0 * c.0).round() as u8,
                                (255.0 * c.1).round() as u8,
                                (255.0 * c.2).round() as u8,
                                255,
                            ],
                        },
                        position.0.round() as i32 + offset_x,
                        position.1.round() as i32 + offset_y,
                        rusttype::Scale::uniform(height),
                        font,
                        &text,
                    );
                }
                crate::video::CompositingMethod::Direct => {
                    imageproc::drawing::draw_text_mut(
                        image,
                        image::Rgba {
                            0: [
                                (255.0 * c.0).round() as u8,
                                (255.0 * c.1).round() as u8,
                                (255.0 * c.2).round() as u8,
                                (255.0 * c.3).round() as u8,
                            ],
                        },
                        position.0.round() as i32 + offset_x,
                        position.1.round() as i32 + offset_y,
                        rusttype::Scale::uniform(height),
                        font,
                        &text,
                    );
                }
                crate::video::CompositingMethod::TransparencySupport => {
                    let max_width = image.width() as i32 - position.0.ceil() as i32;
                    if max_width > 0 {
                        let max_height = image.height() as i32 - position.1.ceil() as i32;
                        if max_height > 0 {
                            let mut img = DynamicImage::new_rgba8(
                                (image.width() as f64 * factor).ceil() as _,
                                (image.height() as f64 * factor).ceil() as _,
                            );
                            imageproc::drawing::draw_text_mut(
                                &mut img,
                                image::Rgba {
                                    0: [
                                        (255.0 * c.0).round() as u8,
                                        (255.0 * c.1).round() as u8,
                                        (255.0 * c.2).round() as u8,
                                        (255.0 * c.3).round() as u8,
                                    ],
                                },
                                offset_x,
                                offset_y,
                                rusttype::Scale::uniform(height),
                                font,
                                &text,
                            );
                            let img = img.as_rgba8().unwrap();
                            let image = image.as_mut_rgba8().unwrap();
                            let width = (img.width() as i32).min(max_width);
                            let height = (img.height() as i32).min(max_height);
                            let x0 = position.0 as i32;
                            let y0 = position.1 as i32;
                            let x1 = if position.0 < 0.0 {
                                (-position.0) as _
                            } else {
                                0
                            };
                            let y1 = if position.1 < 0.0 {
                                (-position.1) as _
                            } else {
                                0
                            };
                            for y in y1..height {
                                for x in x1..width {
                                    crate::video::composite_pixels_transparency_support(
                                        &mut image.get_pixel_mut((x0 + x) as _, (y0 + y) as _).0,
                                        &img.get_pixel(x as _, y as _).0,
                                    );
                                }
                            }
                        }
                    }
                }
                crate::video::CompositingMethod::Manual(_) => {
                    todo!("custom compositing not yet available for text.")
                }
            }
        } else {
            println!("Cannot draw text: No font specified.");
        }
    }
}
