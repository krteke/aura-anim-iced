//! Animated visibility lifecycle management.

use crate::{
    runtime::{Motion, MotionError, MotionEvent, MotionRuntime, PlaybackId},
    timing::Timing,
    traits::{Animatable, IntoMotionAnimation},
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
    exit_playback: Option<PlaybackId>,
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
            exit_playback: None,
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

    /// Updates the target visibility when it differs from the current target.
    ///
    /// Returns `Ok(false)` without starting a new playback when the requested
    /// visibility is already current.
    pub fn set_visible(
        &mut self,
        visible: bool,
        runtime: &mut MotionRuntime,
    ) -> Result<bool, MotionError> {
        if self.shown == visible {
            #[cfg(feature = "tracing")]
            tracing::trace!(
                target: "aura_anim::presence",
                value_type = std::any::type_name::<T>(),
                mounted = self.mounted,
                shown = self.shown,
                "presence visibility is unchanged"
            );
            return Ok(false);
        }

        if visible {
            self.show(runtime)?;
        } else {
            self.hide(runtime)?;
        }
        Ok(true)
    }

    /// Toggles the target visibility using the default transition.
    pub fn toggle(&mut self, runtime: &mut MotionRuntime) -> Result<bool, MotionError> {
        self.set_visible(!self.shown, runtime)
    }

    /// Mounts the content and transitions toward the visible value.
    pub fn show(&mut self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        self.motion
            .transition_to_tracked(self.visible.clone(), runtime)?;
        self.mounted = true;
        self.shown = true;
        self.exit_playback = None;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::presence",
            value_type = std::any::type_name::<T>(),
            mounted = self.mounted,
            shown = self.shown,
            "showing presence"
        );
        Ok(())
    }

    /// Transitions toward the hidden value.
    pub fn hide(&mut self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        let playback = self
            .motion
            .transition_to_tracked(self.hidden.clone(), runtime)?;
        self.shown = false;
        self.exit_playback = Some(playback);
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::presence",
            value_type = std::any::type_name::<T>(),
            mounted = self.mounted,
            shown = self.shown,
            "hiding presence"
        );
        Ok(())
    }

    /// Mounts the content and plays a custom enter animation source.
    pub fn show_with<P, Kind>(
        &mut self,
        animation: P,
        runtime: &mut MotionRuntime,
    ) -> Result<(), MotionError>
    where
        P: IntoMotionAnimation<T, Kind>,
    {
        self.motion.play_tracked(animation, runtime)?;
        self.mounted = true;
        self.shown = true;
        self.exit_playback = None;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::presence",
            value_type = std::any::type_name::<T>(),
            mounted = self.mounted,
            shown = self.shown,
            "showing presence with custom animation"
        );
        Ok(())
    }

    /// Plays a custom exit animation source.
    pub fn hide_with<P, Kind>(
        &mut self,
        animation: P,
        runtime: &mut MotionRuntime,
    ) -> Result<(), MotionError>
    where
        P: IntoMotionAnimation<T, Kind>,
    {
        let playback = self.motion.play_tracked(animation, runtime)?;
        self.shown = false;
        self.exit_playback = Some(playback);
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::presence",
            value_type = std::any::type_name::<T>(),
            mounted = self.mounted,
            shown = self.shown,
            "hiding presence with custom animation"
        );
        Ok(())
    }

    /// Applies one runtime event and returns whether mounted state changed.
    ///
    /// Only completion of the current exit playback unmounts content. Events
    /// from interrupted or superseded exits are ignored.
    pub fn handle_event(&mut self, event: &MotionEvent) -> bool {
        let Some(exit_playback) = self.exit_playback else {
            return false;
        };
        if self.shown || !event.is_completed_for(exit_playback) {
            return false;
        }

        let changed = self.mounted;
        self.mounted = false;
        self.exit_playback = None;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::presence",
            value_type = std::any::type_name::<T>(),
            "unmounted completed hidden presence from runtime event"
        );
        changed
    }

    /// Unmounts hidden content after its animation completes.
    pub fn sync(&mut self, runtime: &MotionRuntime) -> Result<(), MotionError> {
        if !self.shown && self.motion.is_completed(runtime)? {
            self.mounted = false;
            self.exit_playback = None;
            #[cfg(feature = "tracing")]
            tracing::debug!(
                target: "aura_anim::presence",
                value_type = std::any::type_name::<T>(),
                "unmounted completed hidden presence"
            );
        }
        Ok(())
    }
}
