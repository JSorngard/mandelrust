use std::ops::{Add, AddAssign, Mul, MulAssign};

use image::Rgb;
use lazy_static::lazy_static;

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
#[derive(Clone, Copy, Debug, Default)]
pub struct LinearRGB {
    r: f64,
    g: f64,
    b: f64,
}

impl LinearRGB {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        LinearRGB { r, g, b }
    }

    pub fn to_bytes(self) -> [u8; 3] {
        [
            (f64::from(u8::MAX) * self.r.clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * self.g.clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * self.b.clamp(0.0, 1.0)).round() as u8,
        ]
    }
}

impl Add for LinearRGB {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        LinearRGB::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl AddAssign for LinearRGB {
    fn add_assign(&mut self, rhs: Self) {
        *self = {
            Self {
                r: self.r + rhs.r,
                g: self.g + rhs.g,
                b: self.b + rhs.b,
            }
        }
    }
}

impl Mul<f64> for LinearRGB {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        LinearRGB::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl MulAssign<f64> for LinearRGB {
    fn mul_assign(&mut self, rhs: f64) {
        *self = {
            Self {
                r: self.r * rhs,
                g: self.g * rhs,
                b: self.b * rhs,
            }
        }
    }
}

impl From<LinearRGB> for Rgb<u8> {
    /// Converts a `LinearRGB` into an `Rgb<u8>` by converting its
    /// underlying data into the nonlinear sRGB color space.
    /// Clamps the color channels to the range [0, 1] before conversion.
    fn from(linear_rgb: LinearRGB) -> Self {
        Rgb::from([
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.r).clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.g).clamp(0.0, 1.0)).round() as u8,
            (f64::from(u8::MAX) * linear_rgb_to_srgb(linear_rgb.b).clamp(0.0, 1.0)).round() as u8,
        ])
    }
}

impl From<Rgb<f64>> for LinearRGB {
    fn from(srgb: Rgb<f64>) -> Self {
        LinearRGB::new(
            srgb_to_linear_rgb(srgb[0]),
            srgb_to_linear_rgb(srgb[1]),
            srgb_to_linear_rgb(srgb[2]),
        )
    }
}

/// Converts a point in the sRGB color space to a linear RGB triplet.
fn srgb_to_linear_rgb(c: f64) -> f64 {
    c * c // <-- approximation of the below

    //     if c <= 0.04045 {
    //         c / 12.92
    //     } else {
    //         ((c + 0.055) / 1.055).powf(2.4)
    //     }
}

/// Converts an RGB triplet into a point in the sRGB color space.
fn linear_rgb_to_srgb(c: f64) -> f64 {
    c.sqrt() // <-- approximation of the below

    //     if c <= 0.0031308 {
    //         12.92 * c
    //     } else {
    //         1.055 * c.powf(1.0 / 2.4) - 0.055
    //     }
}
