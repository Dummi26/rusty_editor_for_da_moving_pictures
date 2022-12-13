pub enum MightBeRef<'a, T> {
    Owned(T),
    Ref(&'a T),
}
impl<'a, T> MightBeRef<'a, T> {
    pub fn get_ref(&self) -> &T {
        match self {
            Self::Owned(d) => &d,
            Self::Ref(d) => d,
        }
    }
}

impl From<&char> for CharOrAction {
    fn from(ch: &char) -> Self {
        // let mut buf = [0u8; 4]; eprintln!("'{}'\n{:?}", ch, ch.encode_utf8(&mut buf));
        match ch {
            '\n' | '\r' => Self::Enter,
            '\x08' => Self::Backspace,
            '\x7F' => Self::Delete,
            '\t' => Self::Tab,
            '\u{1b}' => Self::Esc,
            ch => Self::Char(ch.clone()),
            _ => Self::Ignored,
        }
    }
}
pub enum CharOrAction {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Tab,
    Ignored,
    Esc,
}

pub fn get_elem_from_index_recursive<'a>(vid: &'a crate::video::Video, index: &mut u32) -> Option<&'a crate::video::Video> {
    if *index == 0 {
        Some(vid)
    } else {
        for child in crate::content::content::Content::children(vid) {
            *index -= 1;
            if let Some(v) = get_elem_from_index_recursive(child, index) { return Some(v); };
        };
        None
    }
}
pub fn get_elem_from_index_recursive_mut<'a>(vid: &'a mut crate::video::Video, index: &mut u32) -> Option<&'a mut crate::video::Video> {
    if *index == 0 {
        Some(vid)
    } else {
        for child in crate::content::content::Content::children_mut(vid) {
            *index -= 1;
            if let Some(v) = get_elem_from_index_recursive_mut(child, index) { return Some(v); };
        };
        None
    }
}