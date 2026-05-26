use std::collections::HashMap;

use super::{AnimationHandle, AnimationPlaybackState, AnimationRegistry};
use crate::{
    property::{PropertySnapshot, PropertyValue, UiProperty, sort_property_entries_by_composition},
    timing::Duration,
};

/// Output produced by one runtime tick.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTick {
    timestamp: Duration,
    properties: PropertySnapshot,
    completed: Vec<AnimationHandle>,
}

impl AnimationTick {
    fn new(
        timestamp: Duration,
        properties: PropertySnapshot,
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

    /// Returns the aggregated property snapshot for view code.
    #[must_use]
    pub fn properties(&self) -> &PropertySnapshot {
        &self.properties
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
    let mut properties_by_key = HashMap::new();
    let mut completed = Vec::new();
    let mut removed = Vec::new();

    for entry in registry.entries_mut().iter_mut() {
        let snapshot = match entry.state() {
            AnimationPlaybackState::Playing => {
                let elapsed = elapsed_since(now, entry.started_at());

                if source_is_complete(entry.source().total_duration(), elapsed) {
                    let snapshot = entry.source().completion_snapshot();

                    entry.set_last_snapshot(snapshot.clone());
                    entry.mark_completed(now);
                    completed.push(entry.handle());
                    removed.push(entry.handle());

                    snapshot
                } else {
                    let snapshot = entry.source().sample_at(elapsed);

                    entry.set_last_snapshot(snapshot.clone());

                    snapshot
                }
            }
            AnimationPlaybackState::Paused => entry.last_snapshot().cloned(),
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

        merge_snapshot(snapshot, &mut properties_by_key);
    }

    for handle in &removed {
        registry.remove(*handle);
    }

    AnimationTick::new(now, sorted_properties(properties_by_key), completed)
}

fn elapsed_since(now: Duration, started_at: Duration) -> Duration {
    now.checked_sub(started_at).unwrap_or(Duration::ZERO)
}

fn source_is_complete(total_duration: Option<Duration>, elapsed: Duration) -> bool {
    total_duration.is_some_and(|duration| elapsed.as_millis() >= duration.as_millis())
}

fn merge_snapshot(
    snapshot: Option<PropertySnapshot>,
    properties_by_key: &mut HashMap<UiProperty, PropertyValue>,
) {
    if let Some(snapshot) = snapshot {
        for (property, value) in snapshot {
            properties_by_key.insert(property, value);
        }
    }
}

fn sorted_properties(properties_by_key: HashMap<UiProperty, PropertyValue>) -> PropertySnapshot {
    let mut properties = properties_by_key.into_iter().collect::<Vec<_>>();

    sort_property_entries_by_composition(&mut properties);

    properties
}
