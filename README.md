# aura-anim-iced

Iced-first animation orchestration for applications that need more than a
single animated value.

This crate builds on Iced's public animation surface instead of replacing it.
User-facing APIs use Iced types such as `iced::Color`, `iced::Vector`,
`iced::Size`, `iced::Rectangle`, `iced::Shadow`, and
`iced::animation::Easing`. Internal interpolation helpers exist only to sample
multi-property keyframes, timelines, runtime snapshots, and diagnostics.

The v0.1 foundation focuses on:

- typed visual properties and sampled property snapshots;
- timing primitives that use Iced easing directly;
- property keyframes and timeline orchestration;
- a runtime that can gate Iced subscriptions while animations are active;
- Iced integration helpers for applying snapshots in `view` code.

Use Iced's `Animation<T>` for direct single-value animation. Use
`aura-anim-iced` when a UI state change needs coordinated opacity, transform,
size, color, shadow, hold, sequence, parallel, and runtime cleanup behavior.
