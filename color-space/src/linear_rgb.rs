use crate::{linear_rgb_to_srgb, quantize_srgb, srgb_to_linear_rgb};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use image::{Luma, Rgb, Rgba};

/// An RGB triplet whose underlying data is not in an sRGB format,
/// but in a linear format. This means that it can be multiplied by a scalar
/// and added to another `LinearRGB`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinearRGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl LinearRGB {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }
}

impl Add for LinearRGB {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl AddAssign for LinearRGB {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Sub for LinearRGB {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}

impl SubAssign for LinearRGB {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
    }
}

impl Mul<f64> for LinearRGB {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl MulAssign<f64> for LinearRGB {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Div<f64> for LinearRGB {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl DivAssign<f64> for LinearRGB {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

impl From<LinearRGB> for Rgb<u8> {
    /// Converts a `LinearRGB` into an `Rgb<u8>` by converting its
    /// underlying data into the nonlinear sRGB color space.
    /// Clamps the color channels to the range \[0, 1\] before conversion.
    fn from(linear_rgb: LinearRGB) -> Self {
        [linear_rgb.r, linear_rgb.g, linear_rgb.b]
            .map(|c| quantize_srgb(linear_rgb_to_srgb(c)))
            .into()
    }
}

impl From<Rgb<f64>> for LinearRGB {
    /// Converts an sRGB triplet into a linear color space where various
    /// transformations are possible.
    fn from(srgb: Rgb<f64>) -> Self {
        let lrgb = srgb.0.map(srgb_to_linear_rgb);
        Self::new(lrgb[0], lrgb[1], lrgb[2])
    }
}

impl From<LinearRGB> for Rgb<f64> {
    fn from(linear_rgb: LinearRGB) -> Self {
        Rgb::from([linear_rgb.r, linear_rgb.g, linear_rgb.b].map(linear_rgb_to_srgb))
    }
}

impl From<[f64; 3]> for LinearRGB {
    fn from(data: [f64; 3]) -> Self {
        Self::new(data[0], data[1], data[2])
    }
}

impl From<LinearRGB> for Luma<u8> {
    fn from(linear_rgb: LinearRGB) -> Self {
        Luma::from([quantize_srgb(linear_rgb_to_srgb(
            linear_rgb.r * 0.2126 + linear_rgb.g * 0.7152 + linear_rgb.b * 0.0722,
        ))])
    }
}

impl From<LinearRGB> for Rgba<u8> {
    fn from(linear_rgb: LinearRGB) -> Self {
        let [r, g, b] = [linear_rgb.r, linear_rgb.g, linear_rgb.b]
            .map(|c| quantize_srgb(linear_rgb_to_srgb(c)));

        [r, g, b, 255].into()
    }
}
