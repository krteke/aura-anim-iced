//! Easing curve definitions and sampling helpers.

/// Describes how normalized animation progress is transformed before sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// A constant-rate easing curve.
    Linear,
}

impl Easing {
    /// Samples this easing curve at normalized progress.
    #[must_use]
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);

        match self {
            Self::Linear => progress,
        }
    }
}
