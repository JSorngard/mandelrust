use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use image::{ColorType, Luma, Rgb, Rgba};

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

/// Maps the range \[0.0, 1.0\] to the range \[0, 255\].
/// Clamps the input to the range before the conversion.
fn quantize_srgb(srgb: f64) -> u8 {
    (f64::from(u8::MAX) * srgb.clamp(0.0, 1.0)).round() as u8
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedPixel<T> {
    Rgba(Rgba<T>),
    Rgb(Rgb<T>),
    Luma(Luma<T>),
}

impl<T> SupportedPixel<T> {
    #[inline]
    pub const fn as_raw(&self) -> &[T] {
        match self {
            Self::Luma(luma) => &luma.0,
            Self::Rgb(rgb) => &rgb.0,
            Self::Rgba(rgba) => &rgba.0,
        }
    }
}

impl From<(SupportedColorType, LinearRGB)> for SupportedPixel<u8> {
    fn from((color_type, linear_rgb): (SupportedColorType, LinearRGB)) -> Self {
        match color_type {
            SupportedColorType::L8 => SupportedPixel::Luma(linear_rgb.into()),
            SupportedColorType::Rgb8 => SupportedPixel::Rgb(linear_rgb.into()),
            SupportedColorType::Rgba8 => SupportedPixel::Rgba(linear_rgb.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedColorType {
    Rgba8,
    Rgb8,
    L8,
}

impl From<SupportedColorType> for ColorType {
    fn from(sct: SupportedColorType) -> Self {
        match sct {
            SupportedColorType::L8 => ColorType::L8,
            SupportedColorType::Rgb8 => ColorType::Rgb8,
            SupportedColorType::Rgba8 => ColorType::Rgba8,
        }
    }
}

impl SupportedColorType {
    pub fn bytes_per_pixel(&self) -> u8 {
        ColorType::from(*self).bytes_per_pixel()
    }

    pub fn has_color(&self) -> bool {
        ColorType::from(*self).has_color()
    }

    pub fn has_alpha(&self) -> bool {
        ColorType::from(*self).has_alpha()
    }

    pub fn channel_count(&self) -> u8 {
        ColorType::from(*self).channel_count()
    }

    pub fn bits_per_pixel(&self) -> u16 {
        ColorType::from(*self).bits_per_pixel()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnsupportedColorTypeError {
    La8,
    L16,
    La16,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
    Unknown,
}

impl std::fmt::Display for UnsupportedColorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is not supported",
            match self {
                Self::La8 => "LA8",
                Self::L16 => "L16",
                Self::La16 => "LA16",
                Self::Rgb16 => "RGB16",
                Self::Rgba16 => "RGBA16",
                Self::Rgb32F => "RGB32F",
                Self::Rgba32F => "RGBA32F",
                Self::Unknown => "<unknown color type>",
            }
        )
    }
}

impl std::error::Error for UnsupportedColorTypeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl TryFrom<ColorType> for SupportedColorType {
    type Error = UnsupportedColorTypeError;
    fn try_from(value: ColorType) -> Result<Self, Self::Error> {
        match value {
            ColorType::L8 => Ok(Self::L8),
            ColorType::Rgb8 => Ok(Self::Rgb8),
            ColorType::Rgba8 => Ok(Self::Rgba8),
            ColorType::La8 => Err(UnsupportedColorTypeError::La8),
            ColorType::L16 => Err(UnsupportedColorTypeError::L16),
            ColorType::La16 => Err(UnsupportedColorTypeError::La16),
            ColorType::Rgb16 => Err(UnsupportedColorTypeError::Rgb16),
            ColorType::Rgba16 => Err(UnsupportedColorTypeError::Rgba16),
            ColorType::Rgb32F => Err(UnsupportedColorTypeError::Rgb32F),
            ColorType::Rgba32F => Err(UnsupportedColorTypeError::Rgba32F),
            _ => Err(UnsupportedColorTypeError::Unknown),
        }
    }
}
