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