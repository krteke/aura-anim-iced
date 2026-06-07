use crate::{
    runtime::{Motion, MotionRuntime},
    timing::Timing,
    traits::{Animatable, Animation},
};

pub struct Presence<T: Animatable> {
    motion: Motion<T>,
    visible: T,
    hidden: T,
    mounted: bool,
    shown: bool,
}

impl<T: Animatable> Presence<T> {
    pub fn new(runtime: &mut MotionRuntime, hidden: T, visible: T, timing: Timing) -> Self {
        Self {
            motion: runtime.motion_with(hidden.clone(), timing),
            visible,
            hidden,
            mounted: false,
            shown: false,
        }
    }

    pub fn motion(&self) -> Motion<T> {
        self.motion
    }

    pub fn value<'a>(&self, runtime: &'a MotionRuntime) -> &'a T {
        self.motion.value_ref(runtime)
    }

    pub const fn is_mounted(&self) -> bool {
        self.mounted
    }

    pub const fn is_visible(&self) -> bool {
        self.shown
    }

    pub fn show(&mut self, runtime: &mut MotionRuntime) {
        self.mounted = true;
        self.shown = true;
        self.motion.transition_to(self.visible.clone(), runtime);
    }

    pub fn hide(&mut self, runtime: &mut MotionRuntime) {
        self.shown = false;
        self.motion.transition_to(self.hidden.clone(), runtime);
    }

    pub fn show_with(&mut self, animation: impl Animation<T>, runtime: &mut MotionRuntime) {
        self.mounted = true;
        self.shown = true;
        self.motion.play(animation, runtime);
    }

    pub fn hide_with(&mut self, animation: impl Animation<T>, runtime: &mut MotionRuntime) {
        self.shown = false;
        self.motion.play(animation, runtime);
    }

    pub fn sync(&mut self, runtime: &MotionRuntime) {
        if !self.shown && self.motion.is_completed(runtime) {
            self.mounted = false;
        }
    }
}
