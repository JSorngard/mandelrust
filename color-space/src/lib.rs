use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use image::Rgb;
use lazy_static::lazy_static;

/// Determines the color of a pixel in linear RGB color space.
/// The color map that this function uses was taken from the python code in
/// [this](https://preshing.com/20110926/high-resolution-mandelbrot-in-obfuscated-python/) blog post.
///
/// As the input increases from 0 to 1 the color transitions as
///
/// black -> brown -> orange -> yellow -> cyan -> blue -> dark blue -> black.
///
/// N.B.: The function has not been tested for inputs outside the range \[0, 1\]
/// and makes no guarantees about the output in that case.
#[inline]
pub fn palette(escape_speed: f64) -> LinearRGB {
    let third_power = escape_speed * escape_speed * escape_speed;
    let ninth_power = third_power * third_power * third_power;
    let eighteenth_power = ninth_power * ninth_power;
    let thirty_sixth_power = eighteenth_power * eighteenth_power;

    LinearRGB::from(
        [
            255.0_f64.powf(-2.0 * ninth_power * thirty_sixth_power) * escape_speed,
            14.0 / 51.0 * escape_speed - 176.0 / 51.0 * eighteenth_power
                + 701.0 / 255.0 * ninth_power,
            16.0 / 51.0 * escape_speed + ninth_power
                - 190.0 / 51.0
                    * thirty_sixth_power
                    * thirty_sixth_power
                    * eighteenth_power
                    * ninth_power,
        ]
        .map(srgb_to_linear_rgb),
    )
}

lazy_static! {
    pub static ref SRGB_TO_LINEAR: Vec<f64> = (0..=u8::MAX)
        .map(|c_srgb| {
            let c_linear = f64::from(c_srgb) / 255.0;
            if c_linear <= 0.04045 {
                c_linear / 12.92
            } else {
                ((c_linear + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    pub static ref LINEAR_TO_SRGB: Vec<f64> = (0..=u8::MAX)
        .map(|c_linear| {
            let c_srgb = f64::from(c_linear) / 255.0;
            if c_srgb <= 0.0031308 {
                12.92 * c_srgb
            } else {
                1.055 * c_srgb.powf(1.0 / 2.4) - 0.055
            }
        })
        .collect();
}

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
        Rgb::from([
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.r).clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.g).clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.b).clamp(0.0, 1.0)).round() as u8,
        ])
    }
}

impl From<Rgb<f64>> for LinearRGB {
    /// Converts an sRGB triplet into a linear color space where various
    /// transformations are possible.
    fn from(srgb: Rgb<f64>) -> Self {
        Self::new(
            srgb_to_linear_rgb(srgb[0]),
            srgb_to_linear_rgb(srgb[1]),
            srgb_to_linear_rgb(srgb[2]),
        )
    }
}

impl From<LinearRGB> for Rgb<f64> {
    fn from(linear_rgb: LinearRGB) -> Self {
        Rgb::from([
            linear_rgb_to_srgb(linear_rgb.r),
            linear_rgb_to_srgb(linear_rgb.g),
            linear_rgb_to_srgb(linear_rgb.b),
        ])
    }
}

impl From<Rgb<u8>> for LinearRGB {
    fn from(srgb: Rgb<u8>) -> Self {
        Self::from(srgb.0.map(|c| SRGB_TO_LINEAR[usize::from(c)]))
    }
}

impl From<[f64; 3]> for LinearRGB {
    fn from(data: [f64; 3]) -> Self {
        Self::new(data[0], data[1], data[2])
    }
}

/// Converts a point in the sRGB color space to a linear RGB triplet.
fn srgb_to_linear_rgb(c: f64) -> f64 {
    c * c // <-- approximation of the below

    // if c <= 0.04045 {
    //     c / 12.92
    // } else {
    //     ((c + 0.055) / 1.055).powf(2.4)
    // }
}

/// Converts an RGB triplet into a point in the sRGB color space.
fn linear_rgb_to_srgb(c: f64) -> f64 {
    c.sqrt() // <-- approximation of the below

    // if c <= 0.0031308 {
    //     12.92 * c
    // } else {
    //     1.055 * c.powf(1.0 / 2.4) - 0.055
    // }
}

#[cfg(test)]
mod test_color_space {
    use super::*;
    use approx::assert_relative_eq;
    use image::Rgb;
    use itertools::Itertools;

    #[test]
    fn check_reversibillity_of_colorspace_conversions() {
        let norm = f64::from(u8::MAX);
        for (r, (g, b)) in
            (0..u8::MAX).cartesian_product((0..=u8::MAX).cartesian_product(0..=u8::MAX))
        {
            let rf = f64::from(r) / norm;
            let gf = f64::from(g) / norm;
            let bf = f64::from(b) / norm;

            let linear_rgb = LinearRGB::new(rf, gf, bf);
            let srgb: Rgb<f64> = linear_rgb.into();
            let after_conversions: LinearRGB = srgb.into();

            assert_relative_eq!(linear_rgb.r, after_conversions.r);
            assert_relative_eq!(linear_rgb.g, after_conversions.g);
            assert_relative_eq!(linear_rgb.b, after_conversions.b);
        }
    }
}
