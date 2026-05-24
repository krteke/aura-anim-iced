#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    pub duration_ms: f32,
}

impl Timing {
    #[must_use]
    pub const fn new(duration_ms: f32) -> Self {
        Self { duration_ms }
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self::new(0.0)
    }
}
