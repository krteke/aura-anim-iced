pub mod handle;
pub mod interpolate;
pub mod keyframes;
pub mod timing;
pub mod traits;
pub mod tween;

const EPSILON_F32: f32 = 1e-5;
const EPSILON_F64: f64 = 1e-10;

pub(crate) fn nearly_equal_f64(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON_F64
}

pub(crate) fn nearly_equal_f32(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON_F32
}
