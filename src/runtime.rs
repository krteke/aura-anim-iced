#[derive(Debug, Default)]
pub struct AnimationRuntime {
    active_count: usize,
}

impl AnimationRuntime {
    #[must_use]
    pub const fn new() -> Self {
        Self { active_count: 0 }
    }

    #[must_use]
    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.active_count == 0
    }
}
