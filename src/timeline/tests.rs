// use float_cmp::assert_approx_eq;

// use super::{
//     Hold, Parallel, Sequence, Timeline, TimelineMarker, TimelinePlayback, TimelinePlaybackState,
//     Track,
// };
// use crate::{
//     keyframes::Keyframes,
//     property::{PropertyValue, UiProperty},
//     timing::{Delay, Duration, Easing, FillMode, IterationCount, Timing},
// };

// fn opacity_track(duration_ms: f64) -> Track {
//     Track::new(
//         Keyframes::new()
//             .with_timing(Timing::new(duration_ms))
//             .opacity(0.0, 0.0)
//             .opacity(1.0, 1.0),
//     )
// }

// fn fixed_opacity_track(duration_ms: f64, from: f32, to: f32) -> Track {
//     Track::new(
//         Keyframes::new()
//             .with_timing(Timing::new(duration_ms))
//             .opacity(0.0, from)
//             .opacity(1.0, to),
//     )
// }

// fn scale_track(duration_ms: f64, from: f32, to: f32) -> Track {
//     Track::new(
//         Keyframes::new()
//             .with_timing(Timing::new(duration_ms))
//             .scale(0.0, from)
//             .scale(1.0, to),
//     )
// }

// fn opacity(snapshot: &[(UiProperty, PropertyValue)]) -> f32 {
//     let Some((_, PropertyValue::Scalar(value))) = snapshot
//         .iter()
//         .find(|(property, _)| *property == UiProperty::Opacity)
//     else {
//         panic!("expected scalar opacity");
//     };

//     *value
// }

// fn scalar(snapshot: &[(UiProperty, PropertyValue)], target: UiProperty) -> f32 {
//     let Some((_, PropertyValue::Scalar(value))) =
//         snapshot.iter().find(|(property, _)| *property == target)
//     else {
//         panic!("expected scalar property");
//     };

//     *value
// }

// #[test]
// fn timeline_starts_empty() {
//     let timeline = Timeline::new();

//     assert_eq!(timeline.name(), None);
//     assert!(timeline.root().steps().is_empty());
//     assert!(timeline.markers().is_empty());
//     assert_approx_eq!(
//         f64,
//         timeline
//             .total_duration()
//             .expect("empty timeline duration")
//             .as_millis(),
//         0.0,
//         epsilon = 1e-10
//     );
//     assert_eq!(timeline.sample_at(Duration::ZERO), None);
// }

// #[test]
// fn tracks_use_keyframe_timing_total_duration() {
//     let track = Track::new(
//         Keyframes::new().with_timing(
//             Timing::new(120.0)
//                 .with_delay(Delay::from_millis(30.0))
//                 .with_iterations(2),
//         ),
//     )
//     .with_name("fade");

//     assert_eq!(track.name(), Some("fade"));
//     assert_approx_eq!(
//         f64,
//         track.total_duration().expect("track duration").as_millis(),
//         270.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn track_builder_helpers_create_keyframe_tracks() {
//     let track = Track::from(UiProperty::Opacity, 0.0)
//         .to(1.0)
//         .duration(Duration::from_millis(100.0))
//         .easing(Easing::EaseIn);

//     assert_approx_eq!(
//         f64,
//         track.total_duration().expect("track duration").as_millis(),
//         100.0,
//         epsilon = 1e-10
//     );

//     let sampled = track
//         .sample_at(Duration::from_millis(50.0))
//         .expect("track sample");

//     assert_approx_eq!(
//         f32,
//         opacity(&sampled),
//         Easing::EaseIn.value(0.5),
//         epsilon = 1e-5
//     );
// }

// #[test]
// fn sequence_duration_sums_step_durations() {
//     let sequence = Sequence::from_steps([
//         opacity_track(100.0).into(),
//         Hold::new(Duration::from_millis(40.0)).into(),
//         opacity_track(60.0).into(),
//     ]);

//     assert_eq!(sequence.steps().len(), 3);
//     assert_approx_eq!(
//         f64,
//         sequence
//             .total_duration()
//             .expect("sequence duration")
//             .as_millis(),
//         200.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn timeline_builder_helpers_compose_sequence_steps() {
//     let timeline = Timeline::track(
//         Track::from(UiProperty::Opacity, 0.0)
//             .to(1.0)
//             .duration(Duration::from_millis(100.0)),
//     )
//     .then(Hold::new(Duration::from_millis(40.0)))
//     .then(
//         Track::from(UiProperty::Scale, 1.0)
//             .to(2.0)
//             .duration(Duration::from_millis(60.0)),
//     );

//     assert_eq!(timeline.root().steps().len(), 3);
//     assert_approx_eq!(
//         f64,
//         timeline
//             .total_duration()
//             .expect("timeline duration")
//             .as_millis(),
//         200.0,
//         epsilon = 1e-10
//     );

//     let sampled = timeline
//         .sample_at(Duration::from_millis(170.0))
//         .expect("timeline sample");

//     assert_approx_eq!(
//         f32,
//         scalar(&sampled, UiProperty::Scale),
//         1.5,
//         epsilon = 1e-5
//     );

//     let sequence_timeline = Timeline::sequence([
//         Hold::new(Duration::from_millis(10.0)).into(),
//         Hold::new(Duration::from_millis(15.0)).into(),
//     ]);

//     assert_approx_eq!(
//         f64,
//         sequence_timeline
//             .total_duration()
//             .expect("sequence timeline duration")
//             .as_millis(),
//         25.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn group_builder_helpers_compose_nested_steps() {
//     let sequence = Sequence::new()
//         .track(
//             Track::from(UiProperty::Opacity, 0.0)
//                 .to(1.0)
//                 .duration(Duration::from_millis(100.0)),
//         )
//         .hold(Duration::from_millis(25.0));
//     let parallel = Parallel::new()
//         .track(
//             Track::from(UiProperty::Scale, 1.0)
//                 .to(2.0)
//                 .duration(Duration::from_millis(100.0)),
//         )
//         .sequence(sequence);
//     let timeline = Timeline::parallel([parallel.into()]);

//     assert_eq!(timeline.root().steps().len(), 1);
//     assert_approx_eq!(
//         f64,
//         timeline
//             .total_duration()
//             .expect("timeline duration")
//             .as_millis(),
//         125.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn parallel_duration_uses_longest_step() {
//     let parallel = Parallel::from_steps([
//         opacity_track(100.0).into(),
//         Hold::new(Duration::from_millis(250.0)).into(),
//         opacity_track(60.0).into(),
//     ]);

//     assert_eq!(parallel.steps().len(), 3);
//     assert_approx_eq!(
//         f64,
//         parallel
//             .total_duration()
//             .expect("parallel duration")
//             .as_millis(),
//         250.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn parallel_sampling_merges_active_track_snapshots() {
//     let parallel = Parallel::from_steps([
//         scale_track(100.0, 1.0, 2.0).into(),
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//     ]);

//     let sampled = parallel
//         .sample_at(Duration::from_millis(50.0))
//         .expect("parallel sample");

//     assert_eq!(
//         sampled
//             .iter()
//             .map(|(property, _)| *property)
//             .collect::<Vec<_>>(),
//         vec![UiProperty::Opacity, UiProperty::Scale]
//     );
//     assert_approx_eq!(
//         f32,
//         scalar(&sampled, UiProperty::Opacity),
//         0.5,
//         epsilon = 1e-5
//     );
//     assert_approx_eq!(
//         f32,
//         scalar(&sampled, UiProperty::Scale),
//         1.5,
//         epsilon = 1e-5
//     );
// }

// #[test]
// fn parallel_sampling_resolves_property_collisions_by_insertion_order() {
//     let parallel = Parallel::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         fixed_opacity_track(100.0, 10.0, 20.0).into(),
//     ]);

//     let sampled = parallel
//         .sample_at(Duration::from_millis(50.0))
//         .expect("parallel sample");

//     assert_eq!(sampled.len(), 1);
//     assert_approx_eq!(f32, opacity(&sampled), 15.0, epsilon = 1e-5);
// }

// #[test]
// fn parallel_sampling_omits_inactive_tracks() {
//     let parallel = Parallel::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         scale_track(200.0, 1.0, 3.0).into(),
//     ]);

//     let sampled = parallel
//         .sample_at(Duration::from_millis(150.0))
//         .expect("parallel sample");

//     assert_eq!(sampled.len(), 1);
//     assert_approx_eq!(
//         f32,
//         scalar(&sampled, UiProperty::Scale),
//         2.5,
//         epsilon = 1e-5
//     );
// }

// #[test]
// fn completion_snapshot_uses_final_visual_state() {
//     let timeline = Timeline::track(
//         Track::from(UiProperty::Opacity, 0.0)
//             .to(1.0)
//             .duration(Duration::from_millis(100.0)),
//     )
//     .then(Hold::new(Duration::from_millis(50.0)));

//     let completed = timeline
//         .completion_snapshot()
//         .expect("completion snapshot after hold");

//     assert_approx_eq!(f32, opacity(&completed), 1.0, epsilon = 1e-5);

//     let parallel = Parallel::new()
//         .track(
//             Track::from(UiProperty::Opacity, 0.0)
//                 .to(1.0)
//                 .duration(Duration::from_millis(100.0)),
//         )
//         .track(
//             Track::from(UiProperty::Scale, 1.0)
//                 .to(2.0)
//                 .duration(Duration::from_millis(100.0)),
//         );
//     let completed_parallel = parallel
//         .completion_snapshot()
//         .expect("parallel completion snapshot");

//     assert_approx_eq!(f32, opacity(&completed_parallel), 1.0, epsilon = 1e-5);
//     assert_approx_eq!(
//         f32,
//         scalar(&completed_parallel, UiProperty::Scale),
//         2.0,
//         epsilon = 1e-5
//     );
// }

// #[test]
// fn timeline_playback_controls_sample_without_runtime_ownership() {
//     let timeline = Timeline::track(
//         Track::from(UiProperty::Opacity, 0.0)
//             .to(1.0)
//             .duration(Duration::from_millis(100.0)),
//     );
//     let mut playback = TimelinePlayback::new();

//     playback.seek(Duration::from_millis(50.0));
//     let playing = playback.snapshot(&timeline);
//     assert_eq!(playing.state(), TimelinePlaybackState::Playing);
//     assert_approx_eq!(f64, playing.position().as_millis(), 50.0, epsilon = 1e-10);
//     assert_approx_eq!(
//         f32,
//         opacity(playing.properties().expect("playing snapshot")),
//         0.5,
//         epsilon = 1e-5
//     );

//     playback.pause();
//     let paused = playback.snapshot(&timeline);
//     assert_eq!(paused.state(), TimelinePlaybackState::Paused);
//     assert_approx_eq!(
//         f32,
//         opacity(paused.properties().expect("paused snapshot")),
//         0.5,
//         epsilon = 1e-5
//     );

//     playback.resume();
//     assert_eq!(playback.state(), TimelinePlaybackState::Playing);

//     playback.cancel();
//     let canceled = playback.snapshot(&timeline);
//     assert_eq!(canceled.state(), TimelinePlaybackState::Canceled);
//     assert_eq!(canceled.properties(), None);

//     playback.seek(Duration::from_millis(25.0));
//     assert_eq!(playback.state(), TimelinePlaybackState::Playing);

//     let finished = playback.finish(&timeline).unwrap();
//     assert_eq!(finished.state(), TimelinePlaybackState::Finished);
//     assert_approx_eq!(f64, finished.position().as_millis(), 100.0, epsilon = 1e-10);
//     assert_approx_eq!(
//         f32,
//         opacity(finished.properties().expect("finished snapshot")),
//         1.0,
//         epsilon = 1e-5
//     );

//     let infinite_timeline = Timeline::track(Track::new(
//         Keyframes::new()
//             .with_timing(Timing::new(100.0).with_iterations(IterationCount::infinite()))
//             .opacity(0.0, 0.0)
//             .opacity(1.0, 1.0),
//     ));
//     let mut playback = TimelinePlayback::new();

//     playback.seek(Duration::from_millis(50.0));
//     assert!(playback.finish(&infinite_timeline).is_err());
// }

// #[test]
// fn timeline_regression_tests_cover_duration_hold_merge_seek_and_completion() {
//     let parallel = Parallel::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         scale_track(100.0, 1.0, 2.0).into(),
//         fixed_opacity_track(100.0, 10.0, 20.0).into(),
//     ]);
//     let timeline = Timeline::sequence([
//         parallel.into(),
//         Hold::new(Duration::from_millis(50.0)).into(),
//     ]);

//     assert_approx_eq!(
//         f64,
//         timeline.root().steps()[0]
//             .total_duration()
//             .expect("parallel duration")
//             .as_millis(),
//         100.0,
//         epsilon = 1e-10
//     );
//     assert_approx_eq!(
//         f64,
//         timeline
//             .total_duration()
//             .expect("sequence duration")
//             .as_millis(),
//         150.0,
//         epsilon = 1e-10
//     );
//     assert_eq!(timeline.sample_at(Duration::from_millis(125.0)), None);

//     let mut playback = TimelinePlayback::new();
//     playback.seek(Duration::from_millis(50.0));
//     let seek = playback.snapshot(&timeline);
//     let seek_properties = seek.properties().expect("seek output");

//     assert_eq!(
//         seek_properties
//             .iter()
//             .map(|(property, _)| *property)
//             .collect::<Vec<_>>(),
//         vec![UiProperty::Opacity, UiProperty::Scale]
//     );
//     assert_approx_eq!(f32, opacity(seek_properties), 15.0, epsilon = 1e-5);
//     assert_approx_eq!(
//         f32,
//         scalar(seek_properties, UiProperty::Scale),
//         1.5,
//         epsilon = 1e-5
//     );

//     let finished = playback.finish(&timeline).expect("finished output");
//     let finished_properties = finished.properties().expect("completion output");

//     assert_eq!(finished.state(), TimelinePlaybackState::Finished);
//     assert_approx_eq!(f32, opacity(finished_properties), 20.0, epsilon = 1e-5);
//     assert_approx_eq!(
//         f32,
//         scalar(finished_properties, UiProperty::Scale),
//         2.0,
//         epsilon = 1e-5
//     );
// }

// #[test]
// fn timeline_duration_uses_root_sequence_and_markers_are_stored() {
//     let mut timeline = Timeline::new().with_name("toast");
//     timeline.push_step(Parallel::from_steps([
//         opacity_track(80.0).into(),
//         opacity_track(120.0).into(),
//     ]));
//     timeline.push_step(Hold::new(Duration::from_millis(300.0)));
//     timeline.push_marker(TimelineMarker::new("settled", Duration::from_millis(120.0)));

//     assert_eq!(timeline.name(), Some("toast"));
//     assert_eq!(timeline.markers()[0].name(), "settled");
//     assert_approx_eq!(
//         f64,
//         timeline.markers()[0].offset().as_millis(),
//         120.0,
//         epsilon = 1e-10
//     );
//     assert_approx_eq!(
//         f64,
//         timeline
//             .total_duration()
//             .expect("timeline duration")
//             .as_millis(),
//         420.0,
//         epsilon = 1e-10
//     );
// }

// #[test]
// fn timeline_marker_helpers_sort_find_and_filter_offsets() {
//     let timeline = Timeline::hold(Duration::from_millis(100.0))
//         .marker("exit", Duration::from_millis(150.0))
//         .marker("mid-a", Duration::from_millis(50.0))
//         .marker("start", Duration::from_millis(-10.0))
//         .marker("mid-b", Duration::from_millis(50.0));

//     assert_eq!(
//         timeline
//             .markers()
//             .iter()
//             .map(TimelineMarker::name)
//             .collect::<Vec<_>>(),
//         vec!["start", "mid-a", "mid-b", "exit"]
//     );
//     assert_approx_eq!(
//         f64,
//         timeline
//             .marker_named("start")
//             .expect("start marker")
//             .offset()
//             .as_millis(),
//         0.0,
//         epsilon = 1e-10
//     );
//     assert_eq!(
//         timeline
//             .marker_named("mid-b")
//             .map(TimelineMarker::offset)
//             .map(Duration::as_millis),
//         Some(50.0)
//     );
//     assert_eq!(timeline.marker_named("missing"), None);
//     assert_eq!(
//         timeline
//             .markers_at_or_before(Duration::from_millis(50.0))
//             .map(TimelineMarker::name)
//             .collect::<Vec<_>>(),
//         vec!["start", "mid-a", "mid-b"]
//     );
//     assert_eq!(
//         timeline
//             .markers_at_or_before(Duration::from_millis(120.0))
//             .map(TimelineMarker::name)
//             .collect::<Vec<_>>(),
//         vec!["start", "mid-a", "mid-b"]
//     );
//     assert_eq!(
//         timeline
//             .markers_at_or_before(Duration::from_millis(200.0))
//             .map(TimelineMarker::name)
//             .collect::<Vec<_>>(),
//         vec!["start", "mid-a", "mid-b", "exit"]
//     );
// }

// #[test]
// fn infinite_track_makes_group_duration_infinite() {
//     let infinite = Track::new(
//         Keyframes::new()
//             .with_timing(Timing::new(100.0).with_iterations(IterationCount::infinite())),
//     );

//     let sequence = Sequence::from_steps([opacity_track(40.0).into(), infinite.clone().into()]);
//     let parallel = Parallel::from_steps([opacity_track(40.0).into(), infinite.into()]);

//     assert_eq!(sequence.total_duration(), None);
//     assert_eq!(parallel.total_duration(), None);
// }

// #[test]
// fn sequence_sampling_advances_through_ordered_steps() {
//     let sequence = Sequence::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         fixed_opacity_track(200.0, 10.0, 20.0).into(),
//     ]);

//     let first = sequence
//         .sample_at(Duration::from_millis(25.0))
//         .expect("first sample");
//     let second = sequence
//         .sample_at(Duration::from_millis(150.0))
//         .expect("second sample");

//     assert_approx_eq!(f32, opacity(&first), 0.25, epsilon = 1e-5);
//     assert_approx_eq!(f32, opacity(&second), 12.5, epsilon = 1e-5);
// }

// #[test]
// fn sequence_sampling_treats_hold_segments_as_silent_time() {
//     let sequence = Sequence::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         Hold::new(Duration::from_millis(50.0)).into(),
//         fixed_opacity_track(100.0, 10.0, 20.0).into(),
//     ]);

//     assert_eq!(sequence.sample_at(Duration::from_millis(125.0)), None);

//     let after_hold = sequence
//         .sample_at(Duration::from_millis(175.0))
//         .expect("sample after hold");

//     assert_approx_eq!(f32, opacity(&after_hold), 12.5, epsilon = 1e-5);
// }

// #[test]
// fn sequence_sampling_uses_next_step_at_boundaries() {
//     let sequence = Sequence::from_steps([
//         fixed_opacity_track(100.0, 0.0, 1.0).into(),
//         fixed_opacity_track(100.0, 10.0, 20.0).into(),
//     ]);

//     let boundary = sequence
//         .sample_at(Duration::from_millis(100.0))
//         .expect("boundary sample");

//     assert_approx_eq!(f32, opacity(&boundary), 10.0, epsilon = 1e-5);
// }

// #[test]
// fn track_sampling_respects_keyframe_timing_fill_mode() {
//     let track = Track::new(
//         Keyframes::new()
//             .with_timing(
//                 Timing::new(100.0)
//                     .with_delay(Delay::from_millis(50.0))
//                     .with_fill_mode(FillMode::Backwards),
//             )
//             .opacity(0.0, 0.0)
//             .opacity(1.0, 1.0),
//     );

//     let before = track
//         .sample_at(Duration::from_millis(25.0))
//         .expect("backwards fill sample");
//     let active = track
//         .sample_at(Duration::from_millis(75.0))
//         .expect("active sample");

//     assert_approx_eq!(f32, opacity(&before), 0.0, epsilon = 1e-5);
//     assert_approx_eq!(f32, opacity(&active), 0.25, epsilon = 1e-5);
// }
