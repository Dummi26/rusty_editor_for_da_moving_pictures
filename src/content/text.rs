use std::{path::PathBuf, io::{self, Read}};

use image::DynamicImage;

use super::content::Content;

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
        Self::new(self.text.clone())
    }

    fn children(&self) -> Vec<&Self> {
        Vec::new()
    }
    fn children_mut(&mut self) -> Vec<&mut Self> {
        Vec::new()
    }

    fn has_changes(&self) -> bool {
        self.as_content_changes.text.is_some() |
        self.as_content_changes.font.is_some() |
        self.as_content_changes.color.is_some()
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

    fn generic_content_data(&mut self) -> &mut super::content::GenericContentData { &mut self.generic_content_data }
}
impl Text {
    pub fn new(text: TextType) -> Self {
        Self { text, font: None, color: crate::types::Color::RGBA(crate::curve::Curve::Constant(1.0), crate::curve::Curve::Constant(1.0), crate::curve::Curve::Constant(1.0), crate::curve::Curve::Constant(1.0)), as_content_changes: TextChanges::default(), generic_content_data: crate::content::content::GenericContentData::default(), }
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
    pub fn text(&self) -> &TextType { &self.text }
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
    pub fn draw(&mut self, image: &mut DynamicImage, prog: f64) {
        let text = self.get_text(prog);
        if let Some(font) = &self.font {
            let c = self.color.get_rgba(prog);
            let dimensions = imageproc::drawing::text_size(rusttype::Scale::uniform(image.height() as f32), font, &text);
            let factor = if dimensions.0 as u32 > image.width() {
                image.width() as f32 / dimensions.0 as f32
            } else { 1.0 };
            imageproc::drawing::draw_text_mut(image, image::Rgba { 0: [
                (255.0 * c.0).round() as u8,
                (255.0 * c.1).round() as u8,
                (255.0 * c.2).round() as u8,
                (255.0 * c.3).round() as u8] }, 0, 0, rusttype::Scale::uniform(image.height() as f32 * factor), font, &text);
        } else {
            println!("Cannot draw text: No font specified.");
        }
    }
}