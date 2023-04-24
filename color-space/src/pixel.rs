use image::{Luma, Rgb, Rgba};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pixel<T> {
    Rgba(Rgba<T>),
    Rgb(Rgb<T>),
    Luma(Luma<T>),
}

impl<T> Pixel<T> {
    #[inline]
    pub const fn as_raw(&self) -> &[T] {
        match self {
            Self::Luma(luma) => &luma.0,
            Self::Rgb(rgb) => &rgb.0,
            Self::Rgba(rgba) => &rgba.0,
        }
    }
}
