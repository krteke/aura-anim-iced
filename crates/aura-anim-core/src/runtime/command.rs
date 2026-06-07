#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationCommand {
    Pause,
    Resume,
    Cancel,
    Seek(f32),
    Finish,
}
