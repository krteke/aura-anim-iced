//! Animated visibility lifecycle management.

use crate::{
    runtime::{Motion, MotionError, MotionRuntime},
    timing::Timing,
    traits::{Animatable, Animation},
};

/// Coordinates mounting and visibility around enter and exit animations.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{MotionRuntime, Presence, timing::Timing};
/// use std::time::Duration;
///
/// let mut runtime = MotionRuntime::new();
/// let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));
///
/// presence.show(&mut runtime).unwrap();
/// assert!(presence.is_mounted());
/// runtime.tick(Duration::from_millis(100));
/// assert_eq!(*presence.value(&runtime).unwrap(), 1.0);
///
/// presence.hide(&mut runtime).unwrap();
/// runtime.tick(Duration::from_millis(100));
/// presence.sync(&runtime).unwrap();
/// assert!(!presence.is_mounted());
/// ```
pub struct Presence<T: Animatable> {
    motion: Motion<T>,
    visible: T,
    hidden: T,
    mounted: bool,
    shown: bool,
}

impl<T: Animatable> Presence<T> {
    /// Creates a hidden, unmounted presence controller.
    #[must_use]
    pub fn new(runtime: &mut MotionRuntime, hidden: T, visible: T, timing: Timing) -> Self {
        Self {
            motion: runtime.motion_with(hidden.clone(), timing),
            visible,
            hidden,
            mounted: false,
            shown: false,
        }
    }

    /// Returns the runtime motion controlled by this presence value.
    pub fn motion(&self) -> Motion<T> {
        self.motion
    }

    /// Returns the current animated value.
    pub fn value<'a>(&self, runtime: &'a MotionRuntime) -> Result<&'a T, MotionError> {
        self.motion.value_ref(runtime)
    }

    /// Returns whether the associated content should be mounted.
    #[must_use]
    pub const fn is_mounted(&self) -> bool {
        self.mounted
    }

    /// Returns whether the presence target is visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.shown
    }

    /// Mounts the content and transitions toward the visible value.
    pub fn show(&mut self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        self.motion.transition_to(self.visible.clone(), runtime)?;
        self.mounted = true;
        self.shown = true;
        Ok(())
    }

    /// Transitions toward the hidden value.
    pub fn hide(&mut self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        self.motion.transition_to(self.hidden.clone(), runtime)?;
        self.shown = false;
        Ok(())
    }

    /// Mounts the content and plays a custom enter animation.
    pub fn show_with(
        &mut self,
        animation: impl Animation<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<(), MotionError> {
        self.motion.play(animation, runtime)?;
        self.mounted = true;
        self.shown = true;
        Ok(())
    }

    /// Plays a custom exit animation.
    pub fn hide_with(
        &mut self,
        animation: impl Animation<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<(), MotionError> {
        self.motion.play(animation, runtime)?;
        self.shown = false;
        Ok(())
    }

    /// Unmounts hidden content after its animation completes.
    pub fn sync(&mut self, runtime: &MotionRuntime) -> Result<(), MotionError> {
        if !self.shown && self.motion.is_completed(runtime)? {
            self.mounted = false;
        }
        Ok(())
    }
}
