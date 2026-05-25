use crate::{property::PropertySnapshot, timing::Timing};

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframes {
    pub frames: Vec<(f32, PropertySnapshot)>,
    pub timing: Timing,
}

impl Keyframes {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            timing: Timing::default(),
        }
    }
}

impl Default for Keyframes {
    fn default() -> Self {
        Self::new()
    }
}
