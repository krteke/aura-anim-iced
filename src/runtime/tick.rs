use super::{AnimationHandle, AnimationPlaybackState};
use crate::runtime::{
    registry::AnimationRegistry,
    target::{AnimationTargetId, TargetedPropertySnapshot},
};
use crate::{property::PropertySnapshot, timing::Duration};

/// Output produced by one runtime tick.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTick {
    timestamp: Duration,
    properties: TargetedPropertySnapshot,
    completed: Vec<AnimationHandle>,
}

impl AnimationTick {
    fn new(
        timestamp: Duration,
        properties: TargetedPropertySnapshot,
        completed: Vec<AnimationHandle>,
    ) -> Self {
        Self {
            timestamp,
            properties,
            completed,
        }
    }

    /// Returns the runtime timestamp used for this tick.
    #[must_use]
    pub const fn timestamp(&self) -> Duration {
        self.timestamp
    }

    /// Returns the target-scoped property snapshots for view code.
    #[must_use]
    pub fn properties(&self) -> &TargetedPropertySnapshot {
        &self.properties
    }

    /// Returns the property snapshot for a single target.
    #[must_use]
    pub fn properties_for(&self, target: AnimationTargetId) -> Option<&PropertySnapshot> {
        self.properties.get(target)
    }

    /// Returns handles completed and removed during this tick.
    #[must_use]
    pub fn completed(&self) -> &[AnimationHandle] {
        &self.completed
    }

    /// Returns whether this tick produced no property output.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

pub(super) fn tick_registry(registry: &mut AnimationRegistry, now: Duration) -> AnimationTick {
    let mut properties = TargetedPropertySnapshot::new();
    let mut completed = Vec::new();
    let mut removed = Vec::new();

    for entry in registry.entries_mut().iter_mut() {
        let target = entry.target();
        let snapshot = match entry.state() {
            AnimationPlaybackState::Playing => {
                let delta = now.checked_sub(entry.last_tick()).unwrap_or(Duration::ZERO);
                entry.update_position(delta);
                entry.set_last_tick(now);

                if let Some(duration) = entry.source().total_duration()
                    && source_is_complete(duration, entry.position())
                {
                    entry.set_position(duration);
                    let snapshot = entry.source().completion_snapshot();

                    entry.set_last_snapshot(snapshot.clone());
                    entry.mark_completed(now);
                    completed.push(entry.handle());
                    removed.push(entry.handle());

                    snapshot
                } else {
                    let snapshot = entry.source().sample_at(entry.position());

                    entry.set_last_snapshot(snapshot.clone());

                    snapshot
                }
            }
            AnimationPlaybackState::Paused => {
                entry.set_last_tick(now);
                entry.last_snapshot().cloned()
            }
            AnimationPlaybackState::Canceled => {
                removed.push(entry.handle());
                None
            }
            AnimationPlaybackState::Completed => {
                completed.push(entry.handle());
                removed.push(entry.handle());
                None
            }
        };

        merge_snapshot(&mut properties, target, snapshot);
    }

    for handle in &removed {
        registry.remove_by_handle(*handle);
    }

    #[cfg(feature = "tracing")]
    tracing::trace!(
        target: "aura_anim_iced::runtime",
        timestamp_ms = now.as_millis(),
        output_targets = properties.targets().len(),
        completed = completed.len(),
        active = registry.active_count(),
        "runtime tick"
    );

    #[cfg(feature = "inspector")]
    tracing::debug!(
        target: "aura_anim_iced::inspector",
        timestamp_ms = now.as_millis(),
        output_targets = properties.targets().len(),
        completed = completed.len(),
        removed = removed.len(),
        active = registry.active_count(),
        "runtime inspector tick"
    );

    AnimationTick::new(now, properties, completed)
}

fn source_is_complete(total_duration: Duration, elapsed: Duration) -> bool {
    elapsed.as_millis() >= total_duration.as_millis()
}

fn merge_snapshot(
    properties: &mut TargetedPropertySnapshot,
    target: AnimationTargetId,
    snapshot: Option<PropertySnapshot>,
) {
    if let Some(snapshot) = snapshot {
        properties.merge(target, snapshot);
    }
}
