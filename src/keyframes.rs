use crate::timing::Timing;

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframes<T> {
    pub frames: Vec<(f32, T)>,
    pub timing: Timing,
}

impl<T> Keyframes<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            timing: Timing::default(),
        }
    }
}

impl<T> Default for Keyframes<T> {
    fn default() -> Self {
        Self::new()
    }
}
