use std::{sync::{RwLock, Arc}};

pub trait Content: Sized {
    fn clone_no_caching(&self) -> Self;
    
    fn children(&self) -> Vec<&Self>;
    fn children_mut(&mut self) -> Vec<&mut Self>;

    fn has_changes(&self) -> bool;
    fn apply_changes(&mut self) -> bool;
    
    fn generic_content_data(&mut self) -> &mut GenericContentData;
}

#[derive(Default)]
pub struct GenericContentData {
    pub highlighted: GenericContentHighlighted,
}

pub enum GenericContentHighlighted {
    No,
    Selected,
}
impl Default for GenericContentHighlighted {
    fn default() -> Self {
        Self::No
    }
}