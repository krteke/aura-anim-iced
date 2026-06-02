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
    removed: Vec<AnimationHandle>,
    scratch: PropertySnapshot,
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
            removed: Vec::new(),
            scratch: PropertySnapshot::new(),
        }
    }

    /// Creates an empty reusable tick output.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Duration::ZERO, TargetedPropertySnapshot::new(), Vec::new())
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
    let mut tick = AnimationTick::empty();

    tick_registry_into(registry, now, &mut tick);

    tick
}

pub(super) fn tick_registry_into(
    registry: &mut AnimationRegistry,
    now: Duration,
    tick: &mut AnimationTick,
) {
    tick.timestamp = now;
    tick.properties.clear();
    tick.completed.clear();
    tick.removed.clear();
    tick.scratch.clear();

    for entry in registry.entries_mut() {
        let target = entry.target();
        match entry.state() {
            AnimationPlaybackState::Playing => {
                let delta = now.checked_sub(entry.last_tick()).unwrap_or(Duration::ZERO);
                entry.update_position(delta);
                entry.set_last_tick(now);

                if let Some(duration) = entry.source().total_duration()
                    && source_is_complete(duration, entry.position())
                {
                    entry.set_position(duration);
                    let has_snapshot = entry.source().completion_snapshot_into(&mut tick.scratch);

                    update_last_snapshot(entry, has_snapshot, &tick.scratch);
                    entry.mark_completed(now);
                    tick.completed.push(entry.handle());
                    tick.removed.push(entry.handle());
                    merge_sampled_snapshot(
                        &mut tick.properties,
                        target,
                        has_snapshot,
                        &tick.scratch,
                    );
                } else {
                    let has_snapshot = entry
                        .source()
                        .sample_into(entry.position(), &mut tick.scratch);

                    update_last_snapshot(entry, has_snapshot, &tick.scratch);
                    merge_sampled_snapshot(
                        &mut tick.properties,
                        target,
                        has_snapshot,
                        &tick.scratch,
                    );
                }
            }
            AnimationPlaybackState::Paused => {
                entry.set_last_tick(now);
                if let Some(snapshot) = entry.last_snapshot() {
                    tick.properties.merge_entries(target, snapshot.entries());
                }
            }
            AnimationPlaybackState::Canceled => {
                tick.removed.push(entry.handle());
            }
            AnimationPlaybackState::Completed => {
                tick.completed.push(entry.handle());
                tick.removed.push(entry.handle());
            }
        }
    }

    tick.scratch.clear();

    for handle in &tick.removed {
        registry.remove_by_handle(*handle);
    }

    #[cfg(feature = "tracing")]
    tracing::trace!(
        target: "aura_anim_iced::runtime",
        timestamp_ms = now.as_millis(),
        output_targets = tick.properties.targets().count(),
        completed = tick.completed.len(),
        active = registry.active_count(),
        "runtime tick"
    );

    #[cfg(feature = "inspector")]
    tracing::debug!(
        target: "aura_anim_iced::inspector",
        timestamp_ms = now.as_millis(),
        output_targets = tick.properties.targets().count(),
        completed = tick.completed.len(),
        removed = tick.removed.len(),
        active = registry.active_count(),
        "runtime inspector tick"
    );
}

fn source_is_complete(total_duration: Duration, elapsed: Duration) -> bool {
    elapsed.as_millis() >= total_duration.as_millis()
}

fn update_last_snapshot(
    entry: &mut crate::runtime::entry::ActiveAnimation,
    has_snapshot: bool,
    snapshot: &PropertySnapshot,
) {
    if has_snapshot {
        entry.replace_last_snapshot(snapshot);
    } else {
        entry.clear_last_snapshot();
    }
}

fn merge_sampled_snapshot(
    properties: &mut TargetedPropertySnapshot,
    target: AnimationTargetId,
    has_snapshot: bool,
    snapshot: &PropertySnapshot,
) {
    if has_snapshot {
        properties.merge_entries(target, snapshot.entries());
    }
}
