use std::num::NonZeroU32;

/// Iteration configuration for a timing value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IterationCount {
    kind: IterationCountKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IterationCountKind {
    Count(NonZeroU32),
    Infinite,
}

impl IterationCount {
    /// A single animation iteration.
    pub const ONCE: Self = Self {
        kind: IterationCountKind::Count(NonZeroU32::MIN),
    };

    /// An infinite number of iterations.
    pub const INFINITE: Self = Self {
        kind: IterationCountKind::Infinite,
    };

    /// Creates a finite iteration count, clamped to at least one iteration.
    #[must_use]
    pub fn count(count: u32) -> Self {
        let count = NonZeroU32::new(count).unwrap_or(NonZeroU32::MIN);

        Self {
            kind: IterationCountKind::Count(count),
        }
    }

    /// Returns an infinite iteration count.
    #[must_use]
    pub const fn infinite() -> Self {
        Self::INFINITE
    }

    /// Returns the finite count when this value is not infinite.
    #[must_use]
    pub const fn finite_count(self) -> Option<u32> {
        match self.kind {
            IterationCountKind::Count(count) => Some(count.get()),
            IterationCountKind::Infinite => None,
        }
    }
}

impl Default for IterationCount {
    fn default() -> Self {
        Self::ONCE
    }
}

impl From<u32> for IterationCount {
    fn from(value: u32) -> Self {
        Self::count(value)
    }
}
