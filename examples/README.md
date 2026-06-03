# aura-anim-iced examples

Run examples from the crate root:

```sh
cargo run --example animated_button
cargo run --example behavior_width
cargo run --example keyframes_popup
cargo run --example route_transition
cargo run --example timeline_toast
cargo run --features palette --example theme_palette_compare
```

## animated_button

Shows a state-driven button transition using parallel timeline tracks for scale,
radius, colors, and shadow.

## behavior_width

Shows a width value change animated through a reusable `BehaviorRule` bound to a
`PropertyTransition`. The controls trigger repeated width changes from the
current rendered width.

## keyframes_popup

Shows direct keyframe sampling for a popup with opacity and scale overshoot.
The close animation tracks its completion handle and hides the popup when the
runtime reports completion.

## route_transition

Shows overlaid horizontal route switching with separate outgoing and incoming
targets. The incoming screen fades in from the right while the outgoing screen
fades out to the left during repeated navigation actions.

## theme_palette_compare

Shows the same theme change animated with ordinary sRGB-alpha interpolation and
with palette-backed Oklab-alpha interpolation so color-heavy theme transitions
can be compared side by side.

## timeline_toast

Shows a full enter, hold, exit lifecycle using a timeline sequence. The example
also declares an example-local `toast-y` property to demonstrate extending the
typed property model without changing the crate.
