#![forbid(unsafe_code)]

/// Determines the color of a pixel in linear RGB color space.
/// The color map that this function uses was taken from the python code in
/// [this](https://preshing.com/20110926/high-resolution-mandelbrot-in-obfuscated-python/) blog post.
///
/// As the input increases from 0 to 1 the color transitions as
///
/// black -> brown -> orange -> yellow -> cyan -> blue -> dark blue -> black.
///
/// # Note
/// The function has not been tested for inputs outside the range \[0, 1\]
/// and makes no guarantees about the output in that case.
#[inline]
pub fn palette(escape_speed: f64) -> LinearRGB {
    let third_power = escape_speed * escape_speed * escape_speed;
    let ninth_power = third_power * third_power * third_power;
    let eighteenth_power = ninth_power * ninth_power;
    let thirty_sixth_power = eighteenth_power * eighteenth_power;

    [
        255.0_f64.powf(-2.0 * ninth_power * thirty_sixth_power) * escape_speed,
        14.0 / 51.0 * escape_speed - 176.0 / 51.0 * eighteenth_power + 701.0 / 255.0 * ninth_power,
        16.0 / 51.0 * escape_speed + ninth_power
            - 190.0 / 51.0
                * thirty_sixth_power
                * thirty_sixth_power
                * eighteenth_power
                * ninth_power,
    ]
    .map(srgb_to_linear_rgb)
    .into()
}

/// Converts a point in the sRGB color space to a linear RGB triplet.
fn srgb_to_linear_rgb(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts an RGB triplet into a point in the sRGB color space.
fn linear_rgb_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Maps the range \[0.0, 1.0\] to the range \[0, 255\].
/// Clamps the input to the range before the conversion.
fn quantize_srgb(srgb: f64) -> u8 {
    (f64::from(u8::MAX) * srgb.clamp(0.0, 1.0)).round() as u8
}

mod linear_rgb;
pub use linear_rgb::LinearRGB;

mod pixel;
pub use pixel::Pixel;

mod supported_color_type;
pub use supported_color_type::{SupportedColorType, UnsupportedColorTypeError};
