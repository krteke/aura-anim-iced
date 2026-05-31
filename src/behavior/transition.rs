use crate::{
    ActivePropertyTransition, AnimationHandle, AnimationRuntime, AnimationTargetId, BehaviorRule,
    Duration, KeyframesBuilder, PropertyTransitionProgress, PropertyTransitionRegistration, Timing,
    behavior::TransitionValueKind, property::PropertySpec, runtime::AnimationClock,
};

/// Tracks one visual property and starts a transition when its target value changes.
///
/// The first observed value becomes the stable target baseline and does not
/// start an animation. Later different values register a two-keyframe animation
/// from the previous visual result to the new target value.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTransition<K: TransitionValueKind>
where
    K::Inner: Copy + PartialEq,
{
    target: AnimationTargetId,
    property: PropertySpec<K>,
    timing: Timing,
    current: Option<K::Inner>,
    active: Option<ActivePropertyTransition<K>>,
}

// TODO: may need optimization
impl<K> PropertyTransition<K>
where
    K: TransitionValueKind,
    K::Inner: Copy + PartialEq,
{
    /// Creates a property transition tracker with default timing.
    #[must_use]
    pub fn new(target: AnimationTargetId, property: PropertySpec<K>) -> Self {
        Self::from_rule(target, &BehaviorRule::new(property))
    }

    /// Creates a property transition tracker from a reusable behavior rule.
    #[must_use]
    pub const fn from_rule(target: AnimationTargetId, rule: &BehaviorRule<K>) -> Self {
        Self {
            target,
            property: rule.property(),
            timing: rule.timing(),
            current: None,
            active: None,
        }
    }

    /// Replaces the timing used for newly registered transitions.
    #[must_use]
    pub const fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    /// Returns the target that receives transition animations.
    #[must_use]
    pub const fn target(&self) -> AnimationTargetId {
        self.target
    }

    /// Returns the tracked property.
    #[must_use]
    pub const fn property(&self) -> PropertySpec<K> {
        self.property
    }

    /// Returns the timing used for newly registered transitions.
    #[must_use]
    pub const fn timing(&self) -> Timing {
        self.timing
    }

    /// Returns the last target value observed by this tracker.
    #[must_use]
    pub const fn current_value(&self) -> Option<K::Inner> {
        self.current
    }

    /// Returns the active runtime handle created by this tracker, if any.
    #[must_use]
    pub const fn active_handle(&self) -> Option<AnimationHandle> {
        match &self.active {
            Some(active) => Some(active.handle()),
            None => None,
        }
    }

    /// Returns whether this tracker currently owns a runtime animation handle.
    #[must_use]
    pub fn is_active<C: AnimationClock>(&self, runtime: &AnimationRuntime<C>) -> bool {
        match &self.active {
            Some(active) => runtime.contains(self.target, active.handle()),
            None => false,
        }
    }

    /// Clears this tracker's active handle when it appears in completed output.
    ///
    /// Returns `true` when this transition handled its own completion.
    pub fn handle_completion<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        let Some(active) = &self.active else {
            return false;
        };

        if runtime.contains(self.target, active.handle()) {
            return false;
        }

        self.active = None;
        true
    }

    /// Returns metadata for the active property transition, if any.
    #[must_use]
    pub const fn active_transition(&self) -> Option<&ActivePropertyTransition<K>> {
        self.active.as_ref()
    }

    /// Returns sampled progress for the active transition at `timestamp`.
    #[must_use]
    pub fn active_progress_at(&self, timestamp: Duration) -> Option<PropertyTransitionProgress<K>> {
        self.active
            .as_ref()
            .map(|active| active.progress_at(timestamp))
    }

    /// Observes a new target value and registers an animation when it changed.
    ///
    /// Returns `None` when the value only seeded the baseline or did not change.
    /// If a previous transition is still running, the replacement starts from
    /// that transition's last sampled visual value.
    pub fn transition_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        value: K::Inner,
    ) -> Option<PropertyTransitionRegistration> {
        self.invalidate_if_stale(runtime);

        let Some(previous) = self.current else {
            self.current = Some(value);
            return None;
        };

        if previous == value {
            return None;
        }

        let from = self.current_visual_value(runtime).unwrap_or(previous);

        Some(self.register_from(runtime, from, value))
    }

    /// Observes a new target value with an explicit current visual value.
    ///
    /// This is useful when application code already stores the rendered value
    /// from the latest tick. The registered animation starts from `visual`
    /// even if the previous target or runtime handle no longer represents what
    /// is currently on screen.
    pub fn transition_from_visual<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        visual: K::Inner,
        value: K::Inner,
    ) -> Option<PropertyTransitionRegistration> {
        self.invalidate_if_stale(runtime);

        if self.current == Some(value) {
            return None;
        }

        self.current = Some(value);

        if visual == value {
            return None;
        }

        Some(self.register_from(runtime, visual, value))
    }

    /// Retargets a currently running transition to a new destination.
    ///
    /// Returns `None` when there is no active runtime animation, the active
    /// handle is stale, or the new destination is already the current target.
    /// The replacement animation starts from the active animation's last
    /// sampled visual value.
    pub fn retarget_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        value: K::Inner,
    ) -> Option<PropertyTransitionRegistration> {
        self.invalidate_if_stale(runtime);

        if self.current == Some(value) {
            return None;
        }

        let from = self.current_visual_value(runtime)?;

        Some(self.register_from(runtime, from, value))
    }

    /// Interrupts the current transition and continues from an explicit visual value.
    ///
    /// Unlike [`transition_from_visual`](Self::transition_from_visual), this
    /// method can replace an active animation even when `value` is already the
    /// current target. This is useful when an external interaction interrupts
    /// playback and application code wants the replacement animation to start
    /// from the exact value currently rendered on screen.
    pub fn interrupt_from_visual<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        visual: K::Inner,
        value: K::Inner,
    ) -> Option<PropertyTransitionRegistration> {
        self.invalidate_if_stale(runtime);

        if self.active.is_none() && self.current == Some(value) {
            return None;
        }

        self.current = Some(value);

        if visual == value {
            if let Some(active) = self.active.take() {
                runtime.cancel(self.target, active.handle());
            }

            return None;
        }

        Some(self.register_from(runtime, visual, value))
    }

    fn register_from<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        from: K::Inner,
        value: K::Inner,
    ) -> PropertyTransitionRegistration {
        let replaced = self.active.take();
        let replaced_handle = replaced.as_ref().map(ActivePropertyTransition::handle);

        if let Some(active) = replaced {
            runtime.cancel(self.target, active.handle());
        }

        self.current = Some(value);

        let registration = runtime.register_keyframes(
            self.target,
            KeyframesBuilder::new()
                .with_timing(self.timing)
                .at(0.0, (self.property, from))
                .at(1.0, (self.property, value))
                .finish(),
        );

        self.active = Some(ActivePropertyTransition::new(
            registration.handle(),
            from,
            value,
            runtime.clock().now(),
            self.timing.total_duration(),
        ));

        PropertyTransitionRegistration::new(registration, replaced_handle)
    }

    fn current_visual_value<C: AnimationClock>(
        &self,
        runtime: &AnimationRuntime<C>,
    ) -> Option<K::Inner> {
        let active = self.active.as_ref()?;
        let snapshot = runtime.last_properties(self.target, active.handle())?;
        let entry = snapshot.find_property(&self.property.raw())?;

        K::unwrap_transition_value(entry.value())
    }

    fn invalidate_if_stale<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) {
        if let Some(active) = &self.active
            && !runtime.contains(self.target, active.handle())
        {
            self.active = None;
        }
    }
}
