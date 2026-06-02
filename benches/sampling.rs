//! Criterion benchmarks for keyframe and timeline sampling paths.

use aura_anim_iced::{
    prelude::*,
    property::{self, PropertyKey},
    runtime::AnimationClock,
};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const SAMPLE_COUNTS: [u64; 3] = [100, 1_000, 10_000];
const TRACK_COUNTS: [usize; 4] = [1, 4, 16, 64];
const OFFSET_COUNTS: [usize; 4] = [2, 8, 32, 128];
const RUNTIME_TARGET_COUNTS: [usize; 3] = [1, 10, 100];
const RUNTIME_ANIMATIONS_PER_TARGET: [usize; 3] = [1, 4, 8];
const BOX_SIZE: PropertySpec<property::Size> =
    PropertySpec::new(PropertyKey::new("bench", "box-size"), 30);
const SCALAR_PROPERTY_NAMES: [&str; 64] = [
    "scalar-00",
    "scalar-01",
    "scalar-02",
    "scalar-03",
    "scalar-04",
    "scalar-05",
    "scalar-06",
    "scalar-07",
    "scalar-08",
    "scalar-09",
    "scalar-10",
    "scalar-11",
    "scalar-12",
    "scalar-13",
    "scalar-14",
    "scalar-15",
    "scalar-16",
    "scalar-17",
    "scalar-18",
    "scalar-19",
    "scalar-20",
    "scalar-21",
    "scalar-22",
    "scalar-23",
    "scalar-24",
    "scalar-25",
    "scalar-26",
    "scalar-27",
    "scalar-28",
    "scalar-29",
    "scalar-30",
    "scalar-31",
    "scalar-32",
    "scalar-33",
    "scalar-34",
    "scalar-35",
    "scalar-36",
    "scalar-37",
    "scalar-38",
    "scalar-39",
    "scalar-40",
    "scalar-41",
    "scalar-42",
    "scalar-43",
    "scalar-44",
    "scalar-45",
    "scalar-46",
    "scalar-47",
    "scalar-48",
    "scalar-49",
    "scalar-50",
    "scalar-51",
    "scalar-52",
    "scalar-53",
    "scalar-54",
    "scalar-55",
    "scalar-56",
    "scalar-57",
    "scalar-58",
    "scalar-59",
    "scalar-60",
    "scalar-61",
    "scalar-62",
    "scalar-63",
];

#[derive(Debug, Clone, Copy)]
struct BenchClock {
    now: Duration,
}

impl AnimationClock for BenchClock {
    fn now(&self) -> Duration {
        self.now
    }
}

fn bench_keyframe_sample_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyframes/sample_counts");
    let keyframes = scalar_keyframes_builder().finish();

    for samples in SAMPLE_COUNTS {
        group.throughput(Throughput::Elements(samples));
        group.bench_with_input(
            BenchmarkId::from_parameter(samples),
            &samples,
            |b, samples| {
                b.iter(|| sample_keyframes_many(black_box(&keyframes), black_box(*samples)));
            },
        );
    }

    group.finish();
}

fn bench_keyframe_value_fixtures(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyframes/value_fixtures");
    let fixtures = [
        ("scalar", scalar_keyframes_builder().finish()),
        ("color", color_keyframes_builder().finish()),
        ("geometry", geometry_keyframes_builder().finish()),
        ("shadow", shadow_keyframes_builder().finish()),
    ];

    for (name, keyframes) in fixtures {
        group.throughput(Throughput::Elements(1_000));
        group.bench_function(name, |b| {
            b.iter(|| sample_keyframes_many(black_box(&keyframes), black_box(1_000)));
        });
    }

    group.finish();
}

fn bench_keyframe_finish_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyframes/finish_matrix");

    for offsets in OFFSET_COUNTS {
        for tracks in TRACK_COUNTS {
            let input_size = offsets * tracks;
            group.throughput(Throughput::Elements(input_size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("offsets_{offsets}"), format!("tracks_{tracks}")),
                &(offsets, tracks),
                |b, &(offsets, tracks)| {
                    b.iter_batched(
                        || scalar_matrix_keyframes_builder(offsets, tracks, false),
                        |builder| black_box(builder.finish()),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.bench_function("duplicate_offsets_32x16", |b| {
        b.iter_batched(
            || scalar_matrix_keyframes_builder(32, 16, true),
            |builder| black_box(builder.finish()),
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_keyframe_sample_track_shapes(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyframes/sample_track_shapes");

    for offsets in OFFSET_COUNTS {
        let keyframes = scalar_matrix_keyframes_builder(offsets, 1, false).finish();

        group.bench_with_input(
            BenchmarkId::new("one_track_offsets", offsets),
            &keyframes,
            |b, keyframes| {
                b.iter(|| sample_keyframes_many(black_box(keyframes), black_box(1_000)));
            },
        );
    }

    for tracks in TRACK_COUNTS {
        let keyframes = scalar_matrix_keyframes_builder(2, tracks, false).finish();

        group.bench_with_input(
            BenchmarkId::new("two_offsets_tracks", tracks),
            &keyframes,
            |b, keyframes| {
                b.iter(|| sample_keyframes_many(black_box(keyframes), black_box(1_000)));
            },
        );
    }

    let keyframes = scalar_matrix_keyframes_builder(32, 16, false).finish();
    for (name, offset) in [
        ("before_first", 0.0),
        ("exact_middle", progress_ratio(16, 31)),
        ("between_middle", 0.515),
        ("after_last", 1.0),
    ] {
        group.bench_function(BenchmarkId::new("position", name), |b| {
            b.iter(|| black_box(keyframes.sample_at(black_box(offset))));
        });
    }

    group.finish();
}

fn bench_mixed_timeline_snapshots(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline/mixed_snapshots");
    let timeline = mixed_timeline();

    for samples in SAMPLE_COUNTS {
        group.throughput(Throughput::Elements(samples));
        group.bench_with_input(
            BenchmarkId::from_parameter(samples),
            &samples,
            |b, samples| {
                b.iter(|| sample_timeline_many(black_box(&timeline), black_box(*samples)));
            },
        );
    }

    group.finish();
}

fn bench_runtime_tick_many_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime/tick_many_targets");

    for targets in RUNTIME_TARGET_COUNTS {
        for animations_per_target in RUNTIME_ANIMATIONS_PER_TARGET {
            let mut runtime = runtime_with_keyframes(targets, animations_per_target, false);
            let input_size = targets * animations_per_target;

            group.throughput(Throughput::Elements(input_size as u64));
            group.bench_function(
                BenchmarkId::new(
                    format!("targets_{targets}"),
                    format!("animations_{animations_per_target}"),
                ),
                |b| {
                    b.iter(|| black_box(runtime.tick()));
                },
            );
        }
    }

    let mut collision_runtime = runtime_with_keyframes(100, 8, true);
    group.bench_function("targets_100_animations_8_collision", |b| {
        b.iter(|| black_box(collision_runtime.tick()));
    });

    group.finish();
}

fn sample_keyframes_many(keyframes: &Keyframes, samples: u64) {
    for index in 0..samples {
        let progress = progress_for(index);
        black_box(keyframes.sample_at(black_box(progress)));
    }
}

fn sample_timeline_many(timeline: &Timeline, samples: u64) {
    for index in 0..samples {
        #[allow(
            clippy::cast_precision_loss,
            reason = "Benchmark sample positions are bounded to 0..1000 before conversion."
        )]
        let offset = Duration::from_millis((index % 1_000) as f64);
        black_box(timeline.sample_at(black_box(offset)));
    }
}

fn scalar_matrix_keyframes_builder(
    offsets: usize,
    tracks: usize,
    duplicate_offsets: bool,
) -> KeyframesBuilder {
    let mut builder = KeyframesBuilder::new().with_timing(Timing::new(1_000.0));

    for offset_index in 0..offsets {
        let progress = progress_ratio(offset_index, offsets.saturating_sub(1));
        let mut snapshot = Vec::with_capacity(tracks);

        for track_index in 0..tracks {
            snapshot.push(property::PropertyEntry::new(
                bench_scalar_spec(track_index),
                scalar_matrix_value(offset_index, track_index),
            ));
        }

        builder.push_at(progress, PropertySnapshot::from(snapshot));

        if duplicate_offsets {
            builder.push_at(
                progress,
                (
                    bench_scalar_spec(0),
                    scalar_matrix_value(offset_index, tracks),
                ),
            );
        }
    }

    builder
}

fn runtime_with_keyframes(
    targets: usize,
    animations_per_target: usize,
    property_collision: bool,
) -> AnimationRuntime<BenchClock> {
    let mut runtime = AnimationRuntime::with_clock(BenchClock {
        now: Duration::from_millis(500.0),
    });

    for _target_index in 0..targets {
        let target = AnimationTargetId::new();

        for animation_index in 0..animations_per_target {
            let property_index = if property_collision {
                0
            } else {
                animation_index
            };
            runtime.register_keyframes(
                target,
                scalar_keyframes_for_property(bench_scalar_spec(property_index)),
            );
        }
    }

    runtime
}

fn scalar_keyframes_for_property(spec: PropertySpec<property::Scalar>) -> Keyframes {
    KeyframesBuilder::new()
        .with_timing(Timing::new(1_000.0).with_easing(Easing::EaseInOut))
        .at(0.0, (spec, 0.0))
        .at(1.0, (spec, 1.0))
        .finish()
}

fn progress_for(index: u64) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "Benchmark sample positions intentionally trade precision for simple distribution."
    )]
    {
        (index % 1_000) as f32 / 999.0
    }
}

fn progress_ratio(index: usize, max_index: usize) -> f32 {
    if max_index == 0 {
        return 0.0;
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "Benchmark matrix dimensions are small and bounded."
    )]
    {
        index as f32 / max_index as f32
    }
}

fn scalar_matrix_value(offset_index: usize, track_index: usize) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "Benchmark matrix dimensions are small and bounded."
    )]
    {
        offset_index as f32 + (track_index as f32 * 0.01)
    }
}

fn bench_scalar_spec(index: usize) -> PropertySpec<property::Scalar> {
    PropertySpec::new(
        PropertyKey::new(
            "bench",
            SCALAR_PROPERTY_NAMES[index % SCALAR_PROPERTY_NAMES.len()],
        ),
        bench_composition_order(index),
    )
}

fn bench_composition_order(index: usize) -> u8 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Benchmark property indexes are wrapped to the static property-name table."
    )]
    {
        (index % SCALAR_PROPERTY_NAMES.len()) as u8
    }
}

fn scalar_keyframes_builder() -> KeyframesBuilder {
    KeyframesBuilder::new()
        .with_timing(Timing::new(1_000.0).with_easing(Easing::EaseInOut))
        .at(
            0.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(OPACITY, 0.0),
                property::PropertyEntry::new(SCALE, 0.95),
                property::PropertyEntry::new(WIDTH, 160.0),
                property::PropertyEntry::new(HEIGHT, 48.0),
            ]),
        )
        .at(
            1.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(OPACITY, 1.0),
                property::PropertyEntry::new(SCALE, 1.05),
                property::PropertyEntry::new(WIDTH, 220.0),
                property::PropertyEntry::new(HEIGHT, 64.0),
            ]),
        )
}

fn color_keyframes_builder() -> KeyframesBuilder {
    KeyframesBuilder::new()
        .with_timing(Timing::new(1_000.0).with_easing(Easing::EaseOut))
        .at(
            0.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(BACKGROUND, iced::Color::from_rgb(0.08, 0.12, 0.16)),
                property::PropertyEntry::new(BORDER_COLOR, iced::Color::from_rgb(0.24, 0.34, 0.46)),
                property::PropertyEntry::new(TEXT_COLOR, iced::Color::from_rgb(0.86, 0.90, 0.94)),
            ]),
        )
        .at(
            1.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(BACKGROUND, iced::Color::from_rgb(0.18, 0.30, 0.42)),
                property::PropertyEntry::new(BORDER_COLOR, iced::Color::from_rgb(0.54, 0.72, 0.90)),
                property::PropertyEntry::new(TEXT_COLOR, iced::Color::WHITE),
            ]),
        )
}

fn geometry_keyframes_builder() -> KeyframesBuilder {
    KeyframesBuilder::new()
        .with_timing(Timing::new(1_000.0))
        .at(
            0.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(TRANSLATE, iced::Vector::new(0.0, 24.0)),
                property::PropertyEntry::new(BOX_SIZE, iced::Size::new(120.0, 44.0)),
            ]),
        )
        .at(
            1.0,
            PropertySnapshot::from(vec![
                property::PropertyEntry::new(TRANSLATE, iced::Vector::new(18.0, 0.0)),
                property::PropertyEntry::new(BOX_SIZE, iced::Size::new(180.0, 64.0)),
            ]),
        )
}

fn shadow_keyframes_builder() -> KeyframesBuilder {
    KeyframesBuilder::new()
        .with_timing(Timing::new(1_000.0).with_easing(Easing::EaseOut))
        .at(0.0, (SHADOW, shadow(0.12, 4.0, 10.0)))
        .at(1.0, (SHADOW, shadow(0.28, 14.0, 32.0)))
}

fn mixed_timeline() -> Timeline {
    let timing = Timing::new(1_000.0).with_easing(Easing::EaseInOut);

    Timeline::parallel([
        Track::new(scalar_keyframes_builder().with_timing(timing).finish()).into(),
        Track::new(color_keyframes_builder().with_timing(timing).finish()).into(),
        Track::new(geometry_keyframes_builder().with_timing(timing).finish()).into(),
        Track::new(shadow_keyframes_builder().with_timing(timing).finish()).into(),
    ])
}

fn shadow(alpha: f32, y: f32, blur: f32) -> iced::Shadow {
    iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, alpha),
        offset: iced::Vector::new(0.0, y),
        blur_radius: blur,
    }
}

criterion_group!(
    benches,
    bench_keyframe_sample_counts,
    bench_keyframe_value_fixtures,
    bench_keyframe_finish_matrix,
    bench_keyframe_sample_track_shapes,
    bench_mixed_timeline_snapshots,
    bench_runtime_tick_many_targets
);
criterion_main!(benches);
