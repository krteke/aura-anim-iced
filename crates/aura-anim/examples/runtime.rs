//! Demonstrates runtime-managed tweens, timelines, keyframes, and presence.

use std::time::Duration;

use aura_anim::prelude::*;

#[derive(Clone, Debug, Animatable)]
struct ButtonMotion {
    opacity: f32,
    scale: f32,
}

#[derive(Clone, Debug, Animatable)]
struct NestedMotion {
    x: f32,
    y: f32,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), MotionError> {
    let mut runtime = MotionRuntime::new();
    let button = runtime.motion_with(
        ButtonMotion {
            opacity: 0.5,
            scale: 0.95,
        },
        Timing::new(160.0).with_easing(Easing::EaseOut),
    );

    button.transition_to(
        ButtonMotion {
            opacity: 1.0,
            scale: 1.0,
        },
        &mut runtime,
    )?;

    runtime.tick(Duration::from_millis(80));
    println!("{:?}", button.value(&runtime).unwrap());

    runtime.tick(Duration::from_millis(80));
    println!("{:?}", button.value(&runtime).unwrap());

    button.play(
        Keyframes::new(button.value(&runtime).unwrap())
            .push(
                80.0,
                ButtonMotion {
                    opacity: 0.7,
                    scale: 1.08,
                },
            )
            .push(
                160.0,
                ButtonMotion {
                    opacity: 1.0,
                    scale: 1.0,
                },
            )
            .with_iterations(IterationCount::INFINITE)
            .with_direction(Direction::Alternate),
        &mut runtime,
    )?;
    runtime.tick(Duration::from_millis(80));
    println!("{:?}", button.value(&runtime).unwrap());

    button.play(
        Spring::new(
            button.value(&runtime).unwrap(),
            ButtonMotion {
                opacity: 0.9,
                scale: 0.9,
            },
            SpringConfig::default(),
        ),
        &mut runtime,
    )?;
    runtime.tick(Duration::from_millis(100));
    button.transition_to(
        ButtonMotion {
            opacity: 1.0,
            scale: 1.0,
        },
        &mut runtime,
    )?;
    runtime.tick(Duration::from_millis(100));
    println!("{:?}", button.value(&runtime).unwrap());

    let start = button.value(&runtime).unwrap();
    let middle = ButtonMotion {
        opacity: 0.5,
        scale: 0.8,
    };
    let end = ButtonMotion {
        opacity: 1.0,
        scale: 1.0,
    };
    button.play(
        Timeline::new(start.clone())
            .then(Tween::between(start, middle.clone(), Timing::new(60.0)))
            .then(Tween::between(middle, end, Timing::new(60.0))),
        &mut runtime,
    )?;
    runtime.tick(Duration::from_millis(60));
    runtime.tick(Duration::from_millis(60));
    println!("{:?}", button.value(&runtime).unwrap());

    let nested_start = NestedMotion { x: 0.0, y: 0.0 };
    let nested = runtime.motion(nested_start.clone());
    let x_branch = Sequence::new(nested_start.clone())
        .then(Tween::between(
            nested_start.clone(),
            NestedMotion { x: 50.0, y: 0.0 },
            Timing::new(80.0),
        ))
        .then(Tween::between(
            NestedMotion { x: 50.0, y: 0.0 },
            NestedMotion { x: 100.0, y: 0.0 },
            Timing::new(80.0),
        ));
    let y_branch = Sequence::new(nested_start.clone())
        .then(Hold::new(nested_start.clone(), Duration::from_millis(40)))
        .then(Tween::between(
            nested_start.clone(),
            NestedMotion { x: 0.0, y: 80.0 },
            Timing::new(120.0),
        ));
    let parallel = Parallel::new(nested_start.clone(), |outputs: &[NestedMotion]| {
        NestedMotion {
            x: outputs[0].x,
            y: outputs[1].y,
        }
    })
    .with(x_branch)
    .with(y_branch);
    nested.play(
        Sequence::new(nested_start)
            .then(parallel)
            .then(Tween::between(
                NestedMotion { x: 100.0, y: 80.0 },
                NestedMotion { x: 0.0, y: 0.0 },
                Timing::new(100.0),
            )),
        &mut runtime,
    )?;
    runtime.tick(Duration::from_millis(260));
    println!("{:?}", nested.value(&runtime).unwrap());

    let retained = runtime.motion_count();
    let transient = runtime.play_once(Tween::between(
        NestedMotion { x: 0.0, y: 0.0 },
        NestedMotion { x: 1.0, y: 1.0 },
        Timing::new(10.0),
    ));
    runtime.tick(Duration::from_millis(10));
    assert!(transient.value(&runtime).is_err());
    assert_eq!(runtime.motion_count(), retained);
    assert_eq!(runtime.active_count(), 0);
    Ok(())
}
