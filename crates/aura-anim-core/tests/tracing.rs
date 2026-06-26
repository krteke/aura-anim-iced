//! Feature-gated verification for emitted tracing events.

#![cfg(feature = "tracing")]

use std::sync::{Arc, Mutex};

use aura_anim_core::{
    binding::MotionBinding,
    presence::Presence,
    runtime::MotionRuntime,
    timing::{Duration, Timing},
};
use tracing::{
    Event, Level, Metadata, Subscriber,
    span::{Attributes, Id, Record},
    subscriber::Interest,
};

#[derive(Clone, Default)]
struct RecordingSubscriber {
    events: Arc<Mutex<Vec<(&'static str, Level)>>>,
}

impl Subscriber for RecordingSubscriber {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let metadata = event.metadata();
        self.events
            .lock()
            .unwrap()
            .push((metadata.target(), *metadata.level()));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}

    fn register_callsite(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,
    Active,
}

#[test]
fn tracing_feature_emits_runtime_binding_and_presence_events() {
    let subscriber = RecordingSubscriber::default();
    let events = Arc::clone(&subscriber.events);

    tracing::subscriber::with_default(subscriber, || {
        let mut runtime = MotionRuntime::new();
        let motion = runtime.motion(0.0_f32);
        motion.transition_to(1.0, &mut runtime).unwrap();
        runtime.tick(Duration::from_millis(16.0));

        let binding = MotionBinding::new(State::Idle, 0.0_f32)
            .when(State::Active, 1.0)
            .fallback(|context| context.tween(Timing::new(100.0)));
        let (bound_motion, mut state) = binding.create_motion(&mut runtime);
        binding
            .set_state(&mut state, State::Active, bound_motion, &mut runtime)
            .unwrap();

        let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));
        presence.show(&mut runtime).unwrap();
    });

    let events = events.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|(target, level)| *target == "aura_anim::runtime" && *level == Level::TRACE)
    );
    assert!(
        events
            .iter()
            .any(|(target, level)| *target == "aura_anim::runtime" && *level == Level::DEBUG)
    );
    assert!(
        events
            .iter()
            .any(|(target, _)| *target == "aura_anim::binding")
    );
    assert!(
        events
            .iter()
            .any(|(target, _)| *target == "aura_anim::presence")
    );
}
