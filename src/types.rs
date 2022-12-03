use crate::curve::Curve;
use std::str::Chars;

pub enum Color {
    RGBA(Curve, Curve, Curve, Curve),
}
impl Color {
    pub fn get_rgba(&self, p: f64) -> (f64, f64, f64, f64) {
        match self {
            Self::RGBA(r, g, b, a) => (r.get_value(p), g.get_value(p), b.get_value(p), a.get_value(p)),
        }
    }

    pub fn parse(chars: &mut Chars) -> Result<Self, crate::files::parser_general::ParserError> {
        Ok(match chars.next() {
            Some('r') => Self::RGBA(crate::files::parser_v0::parse_vid_curve(chars)?, crate::files::parser_v0::parse_vid_curve(chars)?, crate::files::parser_v0::parse_vid_curve(chars)?, crate::files::parser_v0::parse_vid_curve(chars)?),
            Some(c) => return Err(crate::files::parser_general::ParserError::Todo),
            None => return Err(crate::files::parser_general::ParserError::UnexpectedEOF),
        })
    }
}