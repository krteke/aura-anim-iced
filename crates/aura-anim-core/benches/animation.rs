//! Benchmarks for common animation and runtime operations.

use std::hint::black_box;

use aura_anim_core::{
    Animatable, Animation, AnimationCommand, Hold, Interpolate, Motion, MotionRuntime, Parallel,
    Presence, RetainPolicy, Sequence, Spring, SpringConfig, Tween,
    keyframes::Keyframes,
    timing::{Delay, Direction, Duration, Easing, Timing},
};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

#[derive(Clone, Debug, Animatable)]
struct Transform {
    x: f32,
    y: f32,
    scale: f32,
    opacity: f32,
}

#[derive(Clone, Debug, Animatable)]
struct LayoutMetrics {
    origin: (f32, f32),
    size: [f32; 4],
    z_index: i32,
}

fn benchmark_interpolation(criterion: &mut Criterion) {
    let transform_from = Transform {
        x: 0.0,
        y: 20.0,
        scale: 0.8,
        opacity: 0.0,
    };
    let transform_to = Transform {
        x: 300.0,
        y: 120.0,
        scale: 1.2,
        opacity: 1.0,
    };
    let metrics_from = LayoutMetrics {
        origin: (0.0, 20.0),
        size: [64.0, 128.0, 256.0, 512.0],
        z_index: 0,
    };
    let metrics_to = LayoutMetrics {
        origin: (300.0, 120.0),
        size: [96.0, 192.0, 384.0, 768.0],
        z_index: 10,
    };

    let mut group = criterion.benchmark_group("interpolate");

    group.bench_function("f32", |bencher| {
        bencher.iter(|| black_box(f32::interpolate(black_box(&0.0), black_box(&1.0), 0.42)));
    });

    group.bench_function("i32", |bencher| {
        bencher.iter(|| black_box(i32::interpolate(black_box(&0), black_box(&100), 0.42)));
    });

    group.bench_function("tuple4", |bencher| {
        bencher.iter(|| {
            black_box(<(f32, f32, f32, f32)>::interpolate(
                black_box(&(0.0, 10.0, 20.0, 30.0)),
                black_box(&(100.0, 110.0, 120.0, 130.0)),
                black_box(0.42),
            ))
        });
    });

    group.bench_function("array16", |bencher| {
        bencher.iter(|| {
            black_box(<[f32; 16]>::interpolate(
                black_box(&[0.0; 16]),
                black_box(&[100.0; 16]),
                black_box(0.42),
            ))
        });
    });

    group.bench_function("derived_transform", |bencher| {
        bencher.iter(|| {
            black_box(Transform::interpolate(
                black_box(&transform_from),
                black_box(&transform_to),
                black_box(0.42),
            ))
        });
    });

    group.bench_function("derived_nested_metrics", |bencher| {
        bencher.iter(|| {
            black_box(LayoutMetrics::interpolate(
                black_box(&metrics_from),
                black_box(&metrics_to),
                black_box(0.42),
            ))
        });
    });

    group.finish();
}

fn benchmark_timing(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("timing");

    group.bench_function("build_complex_timing", |bencher| {
        bencher.iter(|| {
            black_box(
                Timing::new(black_box(250.0))
                    .with_delay(Delay::from_millis(black_box(40.0)))
                    .with_easing(Easing::EaseOut)
                    .with_direction(Direction::Alternate)
                    .with_iterations(black_box(3)),
            )
        });
    });

    group.bench_function("total_duration", |bencher| {
        let timing = Timing::new(250.0)
            .with_delay(Delay::from_millis(40.0))
            .with_iterations(3);

        bencher.iter(|| black_box(black_box(timing).total_duration()));
    });

    group.finish();
}

#[allow(clippy::too_many_lines)]
fn benchmark_animation_sources(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("animation_source");

    group.bench_function("tween_tick", |bencher| {
        bencher.iter_batched(
            || Tween::between(0.0_f32, 1.0, Timing::new(1_000.0)),
            |mut tween| {
                tween.tick(black_box(Duration::from_millis(16.0)));
                black_box(*tween.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("tween_rate", |bencher| {
        bencher.iter(|| {
            black_box(Tween::between(0.0_f32, 1.0, Timing::new(1_000.0)).rate(black_box(1.5)))
        });
    });

    group.bench_function("tween_seek", |bencher| {
        bencher.iter_batched(
            || {
                Tween::between(
                    0.0_f32,
                    1.0,
                    Timing::new(1_000.0)
                        .with_delay(Delay::from_millis(100.0))
                        .with_easing(Easing::EaseInOut)
                        .with_direction(Direction::Alternate)
                        .with_iterations(3),
                )
            },
            |mut tween| {
                tween.seek(black_box(0.67));
                black_box(*tween.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("tween_transition_to", |bencher| {
        bencher.iter_batched(
            || {
                let mut tween = Tween::between(0.0_f32, 1.0, Timing::new(1_000.0));
                tween.tick(Duration::from_millis(160.0));
                tween
            },
            |mut tween| {
                tween.transition_to(black_box(2.0));
                black_box(*tween.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("keyframes_tick", |bencher| {
        bencher.iter_batched(
            || {
                Keyframes::new(0.0_f32)
                    .push(250.0, 1.0)
                    .push(500.0, 0.5)
                    .push(1_000.0, 1.0)
            },
            |mut keyframes| {
                keyframes.tick(black_box(Duration::from_millis(16.0)));
                black_box(*keyframes.value())
            },
            BatchSize::SmallInput,
        );
    });

    for frame_count in [4, 16, 64] {
        group.bench_with_input(
            BenchmarkId::new("keyframes_tick_many_frames", frame_count),
            &frame_count,
            |bencher, &frame_count| {
                bencher.iter_batched(
                    || keyframes_with_frames(frame_count),
                    |mut keyframes| {
                        keyframes.tick(black_box(Duration::from_millis(379.0)));
                        black_box(*keyframes.value())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("keyframes_seek_many_frames", |bencher| {
        bencher.iter_batched(
            || keyframes_with_frames(64),
            |mut keyframes| {
                keyframes.seek(black_box(0.73));
                black_box(*keyframes.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("sequence_tick", |bencher| {
        bencher.iter_batched(
            || {
                Sequence::new(0.0_f32)
                    .then(Tween::between(0.0, 1.0, Timing::new(250.0)))
                    .then(Tween::between(1.0, 2.0, Timing::new(250.0)))
                    .then(Tween::between(2.0, 3.0, Timing::new(250.0)))
            },
            |mut sequence| {
                sequence.tick(black_box(Duration::from_millis(16.0)));
                black_box(*sequence.value())
            },
            BatchSize::SmallInput,
        );
    });

    for step_count in [4, 16, 64] {
        group.bench_with_input(
            BenchmarkId::new("sequence_tick_many_steps", step_count),
            &step_count,
            |bencher, &step_count| {
                bencher.iter_batched(
                    || sequence_with_steps(step_count),
                    |mut sequence| {
                        sequence.tick(black_box(Duration::from_millis(16.0)));
                        black_box(*sequence.value())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("sequence_seek_many_steps", |bencher| {
        bencher.iter_batched(
            || sequence_with_steps(64),
            |mut sequence| {
                sequence.seek(black_box(0.58));
                black_box(*sequence.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("hold_tick", |bencher| {
        bencher.iter_batched(
            || Hold::new(1.0_f32, Duration::from_millis(1_000.0)),
            |mut hold| {
                hold.tick(black_box(Duration::from_millis(16.0)));
                black_box(*hold.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("hold_seek", |bencher| {
        bencher.iter_batched(
            || Hold::new(1.0_f32, Duration::from_millis(1_000.0)),
            |mut hold| {
                hold.seek(black_box(0.66));
                black_box(*hold.value())
            },
            BatchSize::SmallInput,
        );
    });

    for child_count in [2, 8, 32] {
        group.bench_with_input(
            BenchmarkId::new("parallel_tick", child_count),
            &child_count,
            |bencher, &child_count| {
                bencher.iter_batched(
                    || parallel_with_children(child_count),
                    |mut parallel| {
                        parallel.tick(black_box(Duration::from_millis(16.0)));
                        black_box(*parallel.value())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("parallel_seek", |bencher| {
        bencher.iter_batched(
            || parallel_with_children(32),
            |mut parallel| {
                parallel.seek(black_box(0.42));
                black_box(*parallel.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("spring_tick", |bencher| {
        bencher.iter_batched(
            || Spring::new(0.0_f32, 1.0, SpringConfig::default()),
            |mut spring| {
                spring.tick(black_box(Duration::from_millis(16.0)));
                black_box(*spring.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("spring_retarget", |bencher| {
        bencher.iter_batched(
            || {
                let mut spring = Spring::new(0.0_f32, 1.0, SpringConfig::default());
                spring.tick(Duration::from_millis(160.0));
                spring
            },
            |mut spring| {
                spring.retarget(black_box(2.0));
                black_box(*spring.value())
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

#[allow(clippy::cast_precision_loss)]
fn keyframes_with_frames(frame_count: usize) -> Keyframes<f32> {
    (1..=frame_count).fold(Keyframes::new(0.0_f32), |keyframes, index| {
        let time = index as f64 * 16.0;
        keyframes.push_eased(time, index as f32, Easing::EaseInOut)
    })
}

#[allow(clippy::cast_precision_loss)]
fn sequence_with_steps(step_count: usize) -> Sequence<f32> {
    (0..step_count).fold(Sequence::new(0.0_f32), |sequence, index| {
        sequence.then(Tween::between(
            index as f32,
            (index + 1) as f32,
            Timing::new(16.0),
        ))
    })
}

#[allow(clippy::cast_precision_loss)]
fn parallel_with_children(child_count: usize) -> Parallel<f32> {
    (0..child_count).fold(
        Parallel::new(0.0_f32, |values| values.iter().copied().sum()),
        |parallel, index| {
            parallel.with(Tween::between(
                0.0,
                (index + 1) as f32,
                Timing::new(1_000.0 + index as f64),
            ))
        },
    )
}

fn runtime_with_motions(count: usize) -> (MotionRuntime, Vec<Motion<f32>>) {
    let mut runtime = MotionRuntime::new();
    let motions = (0..count)
        .map(|_| {
            let motion = runtime.motion_with(0.0_f32, Timing::new(1_000.0));
            assert!(motion.transition_to(1.0, &mut runtime).is_ok());
            motion
        })
        .collect();

    (runtime, motions)
}

fn runtime_with_large_motions(count: usize) -> (MotionRuntime, Vec<Motion<[f32; 64]>>) {
    let mut runtime = MotionRuntime::new();
    let motions = (0..count)
        .map(|_| {
            let motion = runtime.motion_with([0.0_f32; 64], Timing::new(1_000.0));
            assert!(motion.transition_to([1.0; 64], &mut runtime).is_ok());
            motion
        })
        .collect();

    (runtime, motions)
}

#[allow(clippy::too_many_lines)]
fn benchmark_runtime(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("runtime");

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("tick_active", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(mut runtime, motions)| {
                        runtime.tick(black_box(Duration::from_millis(16.0)));
                        black_box(motions[0].value(&runtime))
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("insert_and_transition", |bencher| {
        bencher.iter_batched(
            MotionRuntime::new,
            |mut runtime| {
                let motion = runtime.motion_with(0.0_f32, Timing::new(200.0));
                let _ = black_box(motion.transition_to(1.0, &mut runtime));
                black_box(motion)
            },
            BatchSize::SmallInput,
        );
    });

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("value_lookup", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(runtime, motions)| {
                        let mut total = 0.0;
                        for motion in &motions {
                            total += *motion.value_ref(&runtime).unwrap();
                        }
                        black_box(total)
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("pause_resume_active", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(mut runtime, motions)| {
                        for motion in &motions {
                            let _ = black_box(motion.pause(&mut runtime));
                            let _ = black_box(motion.resume(&mut runtime));
                        }
                        black_box(runtime.active_count())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("seek_active", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(mut runtime, motions)| {
                        for motion in &motions {
                            let _ = black_box(motion.seek(0.5, &mut runtime));
                        }
                        black_box(motions[0].value(&runtime))
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("finish_active", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(mut runtime, motions)| {
                        for motion in &motions {
                            let _ = black_box(motion.finish(&mut runtime));
                        }
                        black_box(runtime.active_count())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("finish_active_large_value", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_large_motions(count),
                    |(mut runtime, motions)| {
                        for motion in &motions {
                            let _ = black_box(motion.finish(&mut runtime));
                        }
                        black_box(runtime.active_count())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for count in [1, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("remove", count),
            &count,
            |bencher, &count| {
                bencher.iter_batched(
                    || runtime_with_motions(count),
                    |(mut runtime, motions)| {
                        for motion in motions {
                            let _ = black_box(motion.remove(&mut runtime));
                        }
                        black_box(runtime.motion_count())
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.bench_function("drop_when_settled_tick", |bencher| {
        bencher.iter_batched(
            || {
                let mut runtime = MotionRuntime::new();
                let motion = runtime.insert_with_policy(
                    Tween::between(0.0_f32, 1.0, Timing::new(100.0)),
                    Timing::new(100.0),
                    RetainPolicy::DropWhenSettled,
                );
                (runtime, motion)
            },
            |(mut runtime, motion)| {
                runtime.tick(black_box(Duration::from_millis(100.0)));
                black_box(motion.value(&runtime).is_ok())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("tick_at", |bencher| {
        bencher.iter_batched(
            || {
                let mut runtime = MotionRuntime::new();
                let motion = runtime.motion_with(0.0_f32, Timing::new(1_000.0));
                assert!(motion.transition_to(1.0, &mut runtime).is_ok());
                (runtime, motion, std::time::Instant::now())
            },
            |(mut runtime, motion, now)| {
                runtime.tick_at(black_box(now));
                runtime.tick_at(black_box(now + std::time::Duration::from_millis(16)));
                black_box(motion.value(&runtime).unwrap())
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("command_seek", |bencher| {
        bencher.iter_batched(
            || runtime_with_motions(100),
            |(mut runtime, motions)| {
                for motion in &motions {
                    let _ = black_box(motion.seek(0.25, &mut runtime));
                }
                black_box(motions[0].value(&runtime))
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("runtime_command_seek", |bencher| {
        bencher.iter_batched(
            || runtime_with_motions(100),
            |(mut runtime, motions)| {
                for motion in &motions {
                    let _ = black_box(runtime.command(*motion, AnimationCommand::Seek(0.25)));
                }
                black_box(motions[0].value(&runtime))
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmark_presence(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("presence");

    group.bench_function("show_hide_sync", |bencher| {
        bencher.iter_batched(
            || {
                let mut runtime = MotionRuntime::new();
                let presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));
                (runtime, presence)
            },
            |(mut runtime, mut presence)| {
                presence.show(&mut runtime).unwrap();
                runtime.tick(black_box(Duration::from_millis(100.0)));
                presence.hide(&mut runtime).unwrap();
                runtime.tick(black_box(Duration::from_millis(100.0)));
                presence.sync(&runtime).unwrap();
                black_box((presence.is_mounted(), *presence.value(&runtime).unwrap()))
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("show_with_hide_with", |bencher| {
        bencher.iter_batched(
            || {
                let mut runtime = MotionRuntime::new();
                let presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));
                (runtime, presence)
            },
            |(mut runtime, mut presence)| {
                presence
                    .show_with(
                        Spring::new(0.0_f32, 1.0, SpringConfig::default()),
                        &mut runtime,
                    )
                    .unwrap();
                runtime.tick(black_box(Duration::from_millis(16.0)));
                presence
                    .hide_with(Tween::between(1.0, 0.0, Timing::new(100.0)), &mut runtime)
                    .unwrap();
                runtime.tick(black_box(Duration::from_millis(100.0)));
                presence.sync(&runtime).unwrap();
                black_box((presence.is_visible(), *presence.value(&runtime).unwrap()))
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_interpolation,
    benchmark_timing,
    benchmark_animation_sources,
    benchmark_runtime,
    benchmark_presence
);
criterion_main!(benches);
