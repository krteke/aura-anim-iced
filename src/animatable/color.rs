//! Animation color interpolation support lives in [`crate::color`].

#[cfg(feature = "palette")]
use crate::color::srgba_to_oklaba;
#[cfg(feature = "palette")]
use palette::Oklaba;

use crate::{
    animatable::{Animatable, InterpolationProgress, interpolate_with_progress, lerp_f32_raw},
    color::{AnimColor, Srgba},
};

impl Animatable for AnimColor {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| match (from, to) {
            (Self::Srgba(from), Self::Srgba(to)) => {
                Self::Srgba(interpolate_srgba(from, to, progress))
            }
            #[cfg(feature = "palette")]
            (Self::Oklaba(from), Self::Oklaba(to)) => {
                Self::Oklaba(interpolate_oklaba(from, to, progress))
            }
            #[cfg(feature = "palette")]
            (Self::Srgba(from), Self::Oklaba(to)) => {
                Self::Oklaba(interpolate_oklaba(srgba_to_oklaba(from), to, progress))
            }
            #[cfg(feature = "palette")]
            (Self::Oklaba(from), Self::Srgba(to)) => {
                Self::Oklaba(interpolate_oklaba(from, srgba_to_oklaba(to), progress))
            }
        })
    }
}

fn interpolate_srgba(from: Srgba, to: Srgba, progress: InterpolationProgress) -> Srgba {
    let progress = progress.value();

    Srgba::new(
        lerp_f32_raw(from.red, to.red, progress),
        lerp_f32_raw(from.green, to.green, progress),
        lerp_f32_raw(from.blue, to.blue, progress),
        lerp_f32_raw(from.alpha, to.alpha, progress),
    )
}

#[cfg(feature = "palette")]
fn interpolate_oklaba(from: Oklaba, to: Oklaba, progress: InterpolationProgress) -> Oklaba {
    use palette::Mix;

    from.mix(to, progress.value())
}
