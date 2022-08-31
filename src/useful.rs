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