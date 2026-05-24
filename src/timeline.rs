#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub name: Option<String>,
}

impl Timeline {
    #[must_use]
    pub const fn new() -> Self {
        Self { name: None }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}
