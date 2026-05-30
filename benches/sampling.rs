//! Criterion benchmarks for keyframe and timeline sampling paths.

use aura_anim_iced::{
    BACKGROUND, BORDER_COLOR, Easing, HEIGHT, Keyframes, OPACITY, PropertyKey, PropertySnapshot,
    PropertySpec, SCALE, SHADOW, TEXT_COLOR, TRANSLATE, Timeline, Timing, Track, WIDTH,
    keyframes::KeyframesBuilder, property,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const SAMPLE_COUNTS: [u64; 3] = [100, 1_000, 10_000];
const BOX_SIZE: PropertySpec<property::Size> =
    PropertySpec::new(PropertyKey::new("bench", "box-size"), 30);

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
        let offset = aura_anim_iced::Duration::from_millis((index % 1_000) as f64);
        black_box(timeline.sample_at(black_box(offset)));
    }
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
    bench_mixed_timeline_snapshots
);
criterion_main!(benches);
