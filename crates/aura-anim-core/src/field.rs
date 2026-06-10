//! Type-safe field descriptors and independent struct field animations.

use crate::{Animatable, Animation, AnimationState, IntoMotionAnimation, timing::Duration};

/// Describes an animatable field on `S`.
///
/// `#[derive(Animatable)]` generates one descriptor constant per struct field.
/// Descriptors can also be created manually with [`Field::new`].
pub struct Field<S, V> {
    name: &'static str,
    get: fn(&S) -> &V,
    get_mut: fn(&mut S) -> &mut V,
}

impl<S, V> Copy for Field<S, V> {}

impl<S, V> Clone for Field<S, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, V> Field<S, V> {
    /// Creates a field descriptor from immutable and mutable accessors.
    #[must_use]
    pub const fn new(name: &'static str, get: fn(&S) -> &V, get_mut: fn(&mut S) -> &mut V) -> Self {
        Self { name, get, get_mut }
    }

    /// Returns the field name used to identify duplicate registrations.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    fn get(self, value: &S) -> &V {
        (self.get)(value)
    }

    fn set(self, value: &mut S, field_value: V) {
        *(self.get_mut)(value) = field_value;
    }
}

trait FieldAnimationFactory<S>: 'static {
    fn name(&self) -> &'static str;
    fn build(self: Box<Self>, initial: &S) -> Box<dyn FieldAnimation<S>>;
}

struct TypedFieldAnimationFactory<S, V, Source, Kind> {
    field: Field<S, V>,
    source: Source,
    marker: std::marker::PhantomData<fn() -> Kind>,
}

impl<S, V, Source, Kind> FieldAnimationFactory<S> for TypedFieldAnimationFactory<S, V, Source, Kind>
where
    S: Animatable,
    V: Animatable,
    Source: IntoMotionAnimation<V, Kind> + 'static,
    Kind: 'static,
{
    fn name(&self) -> &'static str {
        self.field.name()
    }

    fn build(self: Box<Self>, initial: &S) -> Box<dyn FieldAnimation<S>> {
        let Self { field, source, .. } = *self;
        let animation = source.into_motion_animation(field.get(initial));

        Box::new(TypedFieldAnimation { field, animation })
    }
}

trait FieldAnimation<S>: 'static {
    fn value_into(&self, output: &mut S);
    fn state(&self) -> AnimationState;
    fn duration(&self) -> Option<Duration>;
    fn advance(&mut self, delta: Duration) -> Duration;
    fn pause(&mut self);
    fn resume(&mut self);
    fn cancel(&mut self);
    fn seek(&mut self, progress: f32);
    fn finish(&mut self);
    fn set_rate(&mut self, rate: f64);
}

struct TypedFieldAnimation<S, V, A> {
    field: Field<S, V>,
    animation: A,
}

impl<S, V, A> FieldAnimation<S> for TypedFieldAnimation<S, V, A>
where
    S: Animatable,
    V: Animatable,
    A: Animation<V>,
{
    fn value_into(&self, output: &mut S) {
        self.field.set(output, self.animation.value().clone());
    }

    fn state(&self) -> AnimationState {
        self.animation.state()
    }

    fn duration(&self) -> Option<Duration> {
        self.animation.duration()
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        self.animation.advance(delta)
    }

    fn pause(&mut self) {
        self.animation.pause();
    }

    fn resume(&mut self) {
        self.animation.resume();
    }

    fn cancel(&mut self) {
        self.animation.cancel();
    }

    fn seek(&mut self, progress: f32) {
        self.animation.seek(progress);
    }

    fn finish(&mut self) {
        self.animation.finish();
    }

    fn set_rate(&mut self, rate: f64) {
        self.animation.set_rate(rate);
    }
}

/// A deferred plan for animating fields of a struct independently.
///
/// The animation factory receives the field's currently sampled value when
/// the plan is played, so interrupted field animations continue without
/// jumping to an earlier origin.
pub struct Fields<S: Animatable> {
    factories: Vec<Box<dyn FieldAnimationFactory<S>>>,
}

impl<S: Animatable> Fields<S> {
    /// Creates an empty field animation plan.
    #[must_use]
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    /// Adds an independently animated field.
    ///
    /// `source` may be an animation, a `|from| ...` factory, or a deferred
    /// target factory such as [`crate::tween_to`] or [`crate::spring_to`].
    /// Registering the same field name again replaces the earlier animation.
    #[must_use]
    pub fn animate<V, Source, Kind>(mut self, field: Field<S, V>, source: Source) -> Self
    where
        V: Animatable,
        Source: IntoMotionAnimation<V, Kind> + 'static,
        Kind: 'static,
    {
        let factory = Box::new(TypedFieldAnimationFactory {
            field,
            source,
            marker: std::marker::PhantomData::<fn() -> Kind>,
        });
        if let Some(existing) = self.factories.iter_mut().find(|f| f.name() == field.name()) {
            *existing = factory;
        } else {
            self.factories.push(factory);
        }
        self
    }

    /// Returns the number of independently animated fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// Returns whether no fields have been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    pub(crate) fn build(self, initial: &S) -> FieldsAnimation<S> {
        let children: Vec<_> = self
            .factories
            .into_iter()
            .map(|factory| factory.build(initial))
            .collect();

        let state = if children.is_empty() {
            AnimationState::Idle
        } else {
            AnimationState::Running
        };

        let mut animation = FieldsAnimation {
            current: initial.clone(),
            children,
            state,
        };
        animation.sample();
        animation
    }
}

impl<S: Animatable> Default for Fields<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an empty field animation plan.
#[must_use]
pub fn fields<S: Animatable>() -> Fields<S> {
    Fields::new()
}

/// An animation that runs independent field animations in parallel.
pub struct FieldsAnimation<S: Animatable> {
    current: S,
    children: Vec<Box<dyn FieldAnimation<S>>>,
    state: AnimationState,
}

impl<S: Animatable> FieldsAnimation<S> {
    fn sample(&mut self) {
        for child in &self.children {
            child.value_into(&mut self.current);
        }
    }
}

impl<S: Animatable> Animation<S> for FieldsAnimation<S> {
    fn value(&self) -> &S {
        &self.current
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        self.children
            .iter()
            .map(|child| child.duration())
            .try_fold(Duration::ZERO, |longest, duration| {
                duration.map(|duration| longest.max(duration))
            })
    }

    fn tick(&mut self, delta: Duration) {
        self.advance(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }
        if self.children.is_empty() {
            self.state = AnimationState::Completed;
            return delta;
        }

        let mut overflow = delta;
        let mut completed = true;
        for child in &mut self.children {
            overflow = overflow.min(child.advance(delta));
            completed &= child.state() == AnimationState::Completed;
        }
        self.sample();

        if completed {
            self.state = AnimationState::Completed;
            overflow
        } else {
            Duration::ZERO
        }
    }

    fn pause(&mut self) {
        if self.state == AnimationState::Running {
            for child in &mut self.children {
                child.pause();
            }
            self.state = AnimationState::Paused;
        }
    }

    fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            for child in &mut self.children {
                child.resume();
            }
            self.state = AnimationState::Running;
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, AnimationState::Running | AnimationState::Paused) {
            for child in &mut self.children {
                child.cancel();
            }
            self.state = AnimationState::Canceled;
        }
    }

    fn seek(&mut self, progress: f32) {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        let duration = self.duration();

        for child in &mut self.children {
            #[allow(clippy::cast_possible_truncation)]
            let child_progress = match (duration, child.duration()) {
                (Some(total), Some(child_duration)) if !child_duration.is_zero() => {
                    (total.as_secs() * f64::from(progress) / child_duration.as_secs())
                        .clamp(0.0, 1.0) as f32
                }
                (Some(_), Some(_)) => 1.0,
                _ => progress,
            };
            child.seek(child_progress);
        }
        self.sample();
        self.state = if progress >= 1.0 {
            AnimationState::Completed
        } else {
            AnimationState::Running
        };
    }

    fn finish(&mut self) {
        for child in &mut self.children {
            child.finish();
        }
        self.sample();
        self.state = AnimationState::Completed;
    }

    fn set_rate(&mut self, rate: f64) {
        for child in &mut self.children {
            child.set_rate(rate);
        }
    }

    fn into_value(self: Box<Self>) -> S {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, fields};
    use crate::{
        Animation, AnimationState, Tween,
        timing::{Duration, Timing},
    };
    use float_cmp::assert_approx_eq;

    #[derive(Clone)]
    struct Position {
        x: f32,
        y: f32,
        fixed: f32,
    }

    impl crate::Interpolate for Position {
        fn interpolate_progress(
            from: &Self,
            to: &Self,
            progress: crate::InterpolationProgress,
        ) -> Self {
            Self {
                x: f32::interpolate_progress(&from.x, &to.x, progress),
                y: f32::interpolate_progress(&from.y, &to.y, progress),
                fixed: f32::interpolate_progress(&from.fixed, &to.fixed, progress),
            }
        }
    }

    const X: Field<Position, f32> = Field::new("x", |value| &value.x, |value| &mut value.x);
    const Y: Field<Position, f32> = Field::new("y", |value| &value.y, |value| &mut value.y);

    #[test]
    fn fields_run_independently_and_preserve_unregistered_values() {
        let initial = Position {
            x: 0.0,
            y: 10.0,
            fixed: 7.0,
        };
        let mut animation = fields()
            .animate(X, |from| Tween::between(from, 100.0, Timing::new(100.0)))
            .animate(Y, |from| Tween::between(from, 210.0, Timing::new(200.0)))
            .build(&initial);

        animation.tick(Duration::from_millis(100.0));

        assert_approx_eq!(f32, animation.value().x, 100.0);
        assert_approx_eq!(f32, animation.value().y, 110.0);
        assert_approx_eq!(f32, animation.value().fixed, 7.0);
        assert_eq!(animation.state(), AnimationState::Running);
    }

    #[test]
    fn duplicate_field_registration_keeps_the_last_animation() {
        let initial = Position {
            x: 0.0,
            y: 0.0,
            fixed: 0.0,
        };
        let mut animation = fields()
            .animate(X, |from| Tween::between(from, 10.0, Timing::new(100.0)))
            .animate(X, |from| Tween::between(from, 20.0, Timing::new(100.0)))
            .build(&initial);

        assert_eq!(animation.children.len(), 1);
        animation.finish();
        assert_approx_eq!(f32, animation.value().x, 20.0);
    }

    #[test]
    fn unregistered_fields_retain_their_original_value() {
        let initial = Position {
            x: 0.0,
            y: 0.0,
            fixed: 7.0,
        };
        let mut animation = fields()
            .animate(Y, |from| Tween::between(from, 10.0, Timing::new(100.0)))
            .build(&initial);

        animation.tick(Duration::from_millis(100.0));
        assert_eq!(animation.state(), AnimationState::Completed);
        assert_approx_eq!(f32, animation.value().x, 0.0);
        assert_approx_eq!(f32, animation.value().y, 10.0);
        assert_approx_eq!(f32, animation.value().fixed, 7.0);
    }
}
