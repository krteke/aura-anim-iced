use uuid::Uuid;

use crate::PropertySnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationTargetId(Uuid);

#[derive(Debug, Clone, PartialEq)]
pub struct TargetedPropertySnapshot {
    targets: Vec<(AnimationTargetId, PropertySnapshot)>,
}

impl AnimationTargetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AnimationTargetId {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetedPropertySnapshot {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn get(&self, target_id: AnimationTargetId) -> Option<&PropertySnapshot> {
        self.targets
            .iter()
            .find(|(id, _)| *id == target_id)
            .map(|(_, snapshot)| snapshot)
    }

    pub fn merge(&mut self, target: AnimationTargetId, snapshot: PropertySnapshot) {
        if let Some(index) = self.targets.iter().position(|(id, _)| *id == target) {
            self.targets[index].1.merge(snapshot);
        } else {
            self.targets.push((target, snapshot));
        }
    }

    pub fn targets(&self) -> &[(AnimationTargetId, PropertySnapshot)] {
        &self.targets
    }
}
