# aura-anim-iced v0.2

## Performance Optimizations

- [x] Store finished keyframes as per-property sample tracks instead of scanning every frame during sampling.
- [x] Add Criterion benchmarks for finish matrices, keyframe sample track shapes, and runtime tick scaling.

## Property Change Animation

- [x] Implement automatic transitions when a visual value changes from one value to another.
- [x] Implement reusable rules that attach animation behavior to value changes.
- [x] Implement smooth continuation when a running value receives a new target.
- [x] Implement value sampling that always starts from the current visual result during changes.
- [x] Implement completion handling for property-driven animations.

## State Driven Animation

- [x] Implement animated transitions between application states.
- [x] Implement state change matching so each state switch can launch the correct animation flow.
- [x] Implement fallback behavior for state switches without a custom transition.
- [x] Implement state transition progress tracking for active animations.
- [x] Implement state completion handling after the visual transition reaches its end.

## Retarget And Interruption

- [x] Implement retargeting for animations that receive a new destination while still running.
- [x] Implement interruption handling that continues from the current visual frame.
- [x] Implement replacement behavior for repeated state changes during the same interaction.
- [x] Implement consistent progress handling when an animation changes direction.
- [x] Implement cleanup for interrupted animations after their replacement starts.

## Route Transition

- [x] Implement reusable page transition behavior for switching between screens.
- [x] Implement outgoing screen animation before the incoming screen reaches its final state.
- [x] Implement incoming screen animation with opacity and position movement.
- [x] Implement route transition state tracking across repeated navigation actions.
- [x] Implement a runnable route transition example that demonstrates screen switching.

## Behavior Example

- [x] Implement a runnable example that animates a changing width value.
- [x] Implement controls that trigger repeated value changes for the example.
- [x] Implement visual output that clearly shows the transition from the current value to the next value.
- [x] Implement example text that explains the animated behavior in plain terms.

## Tests

- [ ] Test automatic transitions after tracked values change.
- [ ] Test retargeting from the current visual result during an active animation.
- [ ] Test interruption behavior during repeated state changes.
- [ ] Test state transition matching across multiple state pairs.
- [ ] Test route transition progress from outgoing screen to incoming screen.

## Documentation

- [ ] Write user-facing documentation for property change animation.
- [ ] Write user-facing documentation for state-driven animation.
- [ ] Write user-facing documentation for retargeting and interruption behavior.
- [ ] Write a short route transition guide with a complete usage flow.
- [ ] Update the project overview to show v0.2 behavior and state animation capabilities.

## Release Preparation

- [ ] Run the full test suite for the v0.2 behavior and state animation work.
- [ ] Run all examples after adding the new behavior and route transition examples.
- [ ] Run formatting and lint checks across the project.
- [ ] Write release notes that summarize the v0.2 functionality.
- [ ] Publish the first v0.2 alpha release after tests, examples, and documentation pass.
