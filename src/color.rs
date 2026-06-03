//! Animation color values with explicit color-space semantics.

use iced::Color;

#[cfg(feature = "palette")]
pub use palette::Oklaba;

/// sRGB color with alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Srgba {
    /// Red channel.
    pub red: f32,
    /// Green channel.
    pub green: f32,
    /// Blue channel.
    pub blue: f32,
    /// Alpha channel.
    pub alpha: f32,
}

impl Srgba {
    /// Creates an sRGB color with alpha.
    #[must_use]
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[cfg(feature = "palette")]
impl From<palette::Srgba> for Srgba {
    fn from(value: palette::Srgba) -> Self {
        Self::new(value.red, value.green, value.blue, value.alpha)
    }
}

impl From<(f32, f32, f32, f32)> for Srgba {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<Color> for Srgba {
    fn from(value: Color) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}

impl From<u32> for Srgba {
    fn from(value: u32) -> Self {
        let bytes = value.to_be_bytes();

        let r = f32::from(bytes[0]) / 255.0;
        let g = f32::from(bytes[1]) / 255.0;
        let b = f32::from(bytes[2]) / 255.0;
        let a = f32::from(bytes[3]) / 255.0;

        Srgba::new(r, g, b, a)
    }
}

impl From<Srgba> for Color {
    fn from(value: Srgba) -> Self {
        Self {
            r: value.red,
            g: value.green,
            b: value.blue,
            a: value.alpha,
        }
    }
}

/// Tags for the color space of an [`AnimColor`].
pub mod tag {
    /// The sRGB color space.
    pub enum Srgba {}

    /// The Oklab color space.
    #[cfg(feature = "palette")]
    pub enum Oklaba {}
}

mod sealed {
    pub trait Sealed {}
}

/// A trait for a color space used by [`AnimColor`].
pub trait ColorSpace: sealed::Sealed {
    /// Wraps an [`Srgba`] value in this color space.
    fn wrap(srgba: Srgba) -> AnimColor;
}

impl sealed::Sealed for tag::Srgba {}

#[cfg(feature = "palette")]
impl sealed::Sealed for tag::Oklaba {}

impl ColorSpace for tag::Srgba {
    fn wrap(s: Srgba) -> AnimColor {
        AnimColor::Srgba(s)
    }
}

#[cfg(feature = "palette")]
impl ColorSpace for tag::Oklaba {
    fn wrap(s: Srgba) -> AnimColor {
        AnimColor::Oklaba(srgba_to_oklaba(s))
    }
}

/// Color value used by animation properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimColor {
    /// sRGB plus alpha. Always available.
    Srgba(Srgba),
    /// Oklab plus alpha. Requires the `palette` feature.
    #[cfg(feature = "palette")]
    Oklaba(Oklaba),
}

impl AnimColor {
    /// Creates an sRGB animation color.
    #[must_use]
    pub const fn srgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::Srgba(Srgba::new(red, green, blue, alpha))
    }

    /// Creates an animation color from a color value in the given color space.
    #[must_use]
    pub fn from_color<C: ColorSpace>(src: impl Into<Srgba>) -> Self {
        C::wrap(src.into())
    }

    /// Creates an Oklab animation color from a hex color value.
    #[cfg(feature = "palette")]
    #[must_use]
    pub fn from_hex<C: ColorSpace>(hex: u32) -> Self {
        C::wrap(hex.into())
    }

    /// Converts this animation color into an Iced color for rendering.
    #[must_use]
    pub fn into_iced(self) -> Color {
        match self {
            Self::Srgba(color) => color.into(),
            #[cfg(feature = "palette")]
            Self::Oklaba(color) => palette_oklaba_to_iced(color),
        }
    }
}

impl From<Color> for AnimColor {
    fn from(value: Color) -> Self {
        Self::Srgba(value.into())
    }
}

impl From<Srgba> for AnimColor {
    fn from(value: Srgba) -> Self {
        Self::Srgba(value)
    }
}

#[cfg(feature = "palette")]
impl From<Oklaba> for AnimColor {
    fn from(value: Oklaba) -> Self {
        Self::Oklaba(value)
    }
}

impl From<AnimColor> for Color {
    fn from(value: AnimColor) -> Self {
        value.into_iced()
    }
}

#[cfg(feature = "palette")]
fn palette_oklaba_to_iced(color: Oklaba) -> Color {
    use palette::{FromColor, Srgba as PaletteSrgba};

    let color = PaletteSrgba::from_color(color);

    Color {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

#[cfg(feature = "palette")]
pub(crate) fn srgba_to_oklaba(color: Srgba) -> Oklaba {
    use palette::{FromColor, Oklaba, Srgba as PaletteSrgba};

    Oklaba::from_color(PaletteSrgba::new(
        color.red,
        color.green,
        color.blue,
        color.alpha,
    ))
}
