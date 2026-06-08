/// A lifecycle command that can be sent to a runtime-managed animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationCommand {
    /// Pause a running animation.
    Pause,
    /// Resume a paused animation.
    Resume,
    /// Cancel an animation.
    Cancel,
    /// Seek to normalized progress.
    Seek(f32),
    /// Move the animation to completion.
    Finish,
}
