use std::any::Any;

use crate::{AnimationCommand, AnimationState, timing::Duration};

pub(super) trait AnimationDyn {
    fn advance(&mut self, delta: Duration);
    fn command(&mut self, command: AnimationCommand);
    fn compact(&mut self);
    fn is_active(&self) -> bool;
    fn state(&self) -> AnimationState;
    fn value_any(&self) -> &dyn Any;
    fn retarget_any(&mut self, target: &dyn Any) -> bool;
}
