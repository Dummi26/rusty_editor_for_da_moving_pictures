use crate::project::Project;

pub trait Content: Sized {
    fn clone_no_caching(&self) -> Self;

    fn children(&self) -> Vec<&Self>;
    fn children_mut(&mut self) -> Vec<&mut Self>;

    fn has_changes(&self) -> bool;
    fn apply_changes(&mut self) -> bool;

    fn generic_content_data(&mut self) -> &mut GenericContentData;
}

#[derive(Clone)]
pub struct GenericContentData {
    pub project: Project,
    pub highlighted: GenericContentHighlighted,
}

impl GenericContentData {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            highlighted: GenericContentHighlighted::No,
        }
    }
    pub fn reset(&self) -> Self {
        Self::new(self.project.clone())
    }
}

#[derive(Clone)]
pub enum GenericContentHighlighted {
    No,
    Selected,
}
