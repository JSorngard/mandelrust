#![forbid(unsafe_code)]

mod u32_and_usize;

use core::num::{NonZeroU32, NonZeroU8, TryFromIntError};
use std::io::Write;

use image::{DynamicImage, ImageBuffer, Luma, Rgb, Rgba};
use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    prelude::ParallelSliceMut,
};

use color_space::{palette, LinearRGB, Pixel, SupportedColorType};
pub use u32_and_usize::U32AndUsize;

// ----------- DEBUG FLAGS --------------
// Set to true to only super sample close to the border of the set.
const RESTRICT_SSAA_REGION: bool = true;

// Supersampling will be aborted if the escape speed of a point is larger than this.
// For low enough resolutions this region will begin clipping into the
// fractal, but for typical image resolutions this is not an issue.
const SSAA_REGION_CUTOFF: f64 = 0.963;

// Set to true to display the region where supersampling is not done
// as orange/brown. The border region where supersampling is only partially done
// will appear as black.
const SHOW_SSAA_REGION: bool = false;

// Set to false to not mirror the image.
// Only relevant when the image contains the real axis.
const ENABLE_MIRRORING: bool = true;

// If false the program iterates all pixels in the cardioid and period 2 bulb.
// If true a check is performed for every pixel to determine whether they
// are in those regions without iterating.
// Could be faster to disable this if you will be looking only at regions where
// these features are not visible.
const CARDIOID_AND_BULB_CHECK: bool = true;
// --------------------------------------

/// Takes in variables describing where to render and at what resolution
/// and produces an image of the Mandelbrot set.
///
/// `render_parameters` contains `x_resolution`, `y_resolution`, `max_iterations`, `sqrt_samples_per_pixel` and `grayscale`.
///
/// `render_region` contains `center_real`, `center_imag`, `real_distance` and `imag_distance`.
///
/// `x_resolution` and `y_resolution` is the resolution in pixels in the real
/// and imaginary direction respectively.
/// `sqrt_samples_per_pixel` is the number of supersampled points along one direction. If it
/// is e.g. 3, then a supersampled pixel will be sampled 3^2 = 9 times.
///
/// `center_real` and `center_imag` are the real and imaginary parts of the
/// point at the center of the image.
///
/// `real_distance` and `imag_distance` describe the size of the region in the
/// complex plane to render.
///
/// ```text
///           real_distance
/// |-------------------------------|
/// |                               |
/// |              x                |  imag_distance
/// |  center_real + center_imag*i  |
/// |-------------------------------|
/// ```
/// If `real_distance` = `imag_distance` = 1,
/// `x_resolution` = `y_resolution` = 100 and `center_real` = `center_imag` = 0 a square
/// of size 1x1 centered on the origin will be computed and rendered as a
/// 100x100 pixel image.
///
/// `max_iterations` is the maximum number of iterations to compute for each pixel sample before labeling
/// a point as part of the set.
///
/// If `grayscale` is true the image is rendered in grayscale instead of color.
///
/// If `verbose` is true the function will use prints to `stderr` to display a progress bar.
#[must_use]
pub fn render(
    render_parameters: RenderParameters,
    render_region: Frame,
    verbose: bool,
) -> DynamicImage {
    let x_resolution = render_parameters.x_resolution;
    let y_resolution = render_parameters.y_resolution;
    let color_type = render_parameters.color_type;

    // We store the pixel data in a rotated fashion so that
    // the data for pixels along the y-axis lie contiguous in memory.
    let mut image = match color_type {
        SupportedColorType::L8 => DynamicImage::ImageLuma8(
            // That is the reason for the switched dimensions in these calls to `new`.
            ImageBuffer::<Luma<u8>, Vec<u8>>::new(y_resolution.into(), x_resolution.into()),
        ),
        SupportedColorType::Rgb8 => DynamicImage::ImageRgb8(ImageBuffer::<Rgb<u8>, Vec<u8>>::new(
            y_resolution.into(),
            x_resolution.into(),
        )),
        SupportedColorType::Rgba8 => DynamicImage::ImageRgba8(
            ImageBuffer::<Rgba<u8>, Vec<u8>>::new(y_resolution.into(), x_resolution.into()),
        ),
    };

    let progress_bar = if verbose {
        ProgressBar::new(x_resolution.into())
    } else {
        ProgressBar::hidden()
    };

    match &mut image {
        DynamicImage::ImageLuma8(buffer) => buffer.as_mut(),
        DynamicImage::ImageRgb8(buffer) => buffer.as_mut(),
        DynamicImage::ImageRgba8(buffer) => buffer.as_mut(),
        _ => unreachable!("we define the image so that it can only be one of the above"),
    }
    // Split the image up into vertical bands and iterate over them in parallel.
    .par_chunks_exact_mut(usize::from(color_type.bytes_per_pixel()) * usize::from(y_resolution))
    // We enumerate each band to be able to compute the real value of c for that band.
    .enumerate()
    .progress_with(progress_bar)
    .for_each(|(band_index, band)| color_band(render_parameters, render_region, band_index, band));

    if verbose {
        // Attempt to report progress, but if this fails it's not important and we just continue.
        _ = write!(std::io::stdout(), "\rProcessing image");
        _ = std::io::stdout().flush();
    }

    // Undo the rotated state used during rendering.
    image.rotate270()
}

/// Computes the colors of the pixels in a y-axis band of the image of the mandelbrot set.
fn color_band(
    render_parameters: RenderParameters,
    render_region: Frame,
    band_index: usize,
    band: &mut [u8],
) {
    let x_resolution_f64 = f64::from(render_parameters.x_resolution);
    let y_resolution_f64 = f64::from(render_parameters.y_resolution);

    let mut mirror_from: usize = 0;
    let real_delta = render_region.real_distance / (x_resolution_f64 - 1.0);
    let imag_delta = render_region.imag_distance / (y_resolution_f64 - 1.0);

    // True if the image contains the real axis, false otherwise.
    // If the image contains the real axis we want to mirror
    // the result of the largest half on to the smallest.
    let mirror = ENABLE_MIRRORING && render_region.center_imag.abs() < render_region.imag_distance;
    let start_real = render_region.center_real - render_region.real_distance / 2.0;

    // One way of doing this is to always assume that the half with negative
    // imaginary part is the larger one. If the assumption is false
    // we only need to flip the image vertically to get the
    // correct result since it is symmetric under conjugation.
    let need_to_flip = render_region.center_imag > 0.0;
    let start_imag = if need_to_flip { -1.0 } else { 1.0 } * render_region.center_imag
        - render_region.imag_distance / 2.0;

    // This is the real value of c for this entire band.
    let c_real = start_real + render_region.real_distance * (band_index as f64) / x_resolution_f64;

    let bytes_per_pixel = usize::from(render_parameters.color_type.bytes_per_pixel());

    for y_index in (0..band.len()).step_by(bytes_per_pixel) {
        // Compute the imaginary part at this pixel
        let c_imag = start_imag
            + render_region.imag_distance * (y_index as f64)
                / (bytes_per_pixel as f64 * y_resolution_f64);

        if !(mirror && c_imag > 0.0) {
            let pixel_region = Frame::new(c_real, c_imag, real_delta, imag_delta);

            // Compute the pixel color as normal by iteration
            let color = pixel_color(pixel_region, render_parameters);

            // and `memcpy` it to the correct place.
            band[y_index..(bytes_per_pixel + y_index)].copy_from_slice(color.as_raw());

            // We keep track of how many pixels have been colored
            // in order to potentially mirror them.
            mirror_from += bytes_per_pixel;
        } else {
            // We have rendered every pixel with negative imaginary part.

            // We want to mirror from the next pixel over every iteration.
            // This line of code is before the mirroring since the first time
            // we enter this branch the pixel indicated by `mirror_from` is
            // the one that contains the real line, and we do not want to
            // mirror that one since the real line is infinitely thin.
            mirror_from -= bytes_per_pixel;

            // `memmove` the data from the already computed pixel into this one.
            band.copy_within((mirror_from - bytes_per_pixel)..mirror_from, y_index);
        }
    }

    // If our assumption that we are rendering in the region of the complex plane with
    // negative imaginary component is false we must flip the vertical band
    // to get the correct image.
    if need_to_flip {
        // Flip all data in the band. Turns RGB(A) into (A)BGR.
        band.reverse();

        if bytes_per_pixel > 1 {
            for pixel in band.chunks_exact_mut(bytes_per_pixel) {
                // Flip each pixel from (A)BGR to RGB(A).
                pixel.reverse();
            }
        }
    }
}

/// Computes the escape speed for samples in a grid inside
/// the pixel region, works out the color of each sample and
/// returns the average color as an sRGB value. If x is the center
/// of the pixel region and `sqrt_samples_per_pixel` = 3,
/// then the dots are also sampled:
///
/// ```text
///  real_distance
///    -------
///    .  .  .  |
///    .  x  .  | imag_distance
///    .  .  .  |
/// ```
///
/// The gap between the sample points at the edge and the
/// edge of the pixel is the same as between the points.
///
/// N.B.: if `render_parameters.sqrt_samples_per_pixel` is even the center of
/// the pixel is never sampled, and if it is 1 no super
/// sampling is done (only the center is sampled).
fn pixel_color(pixel_region: Frame, render_parameters: RenderParameters) -> Pixel<u8> {
    let ssaa = render_parameters.sqrt_samples_per_pixel.get();
    let ssaa_f64: f64 = ssaa.into();

    // `samples` can be a u16 since the maximum number of samples is u8::MAX^2 which is less than u16::MAX
    let mut samples: u16 = 0;
    let max_samples: usize = usize::from(ssaa) * usize::from(ssaa);

    // Initialize the pixel color as black.
    let mut color = LinearRGB::default();

    // Supersampling loop.
    for (i, j) in (1..=ssaa)
        .cartesian_product(1..=ssaa)
        // We start the super sampling loop in the middle in order to ensure
        // that if we abort supersampling, we have sampled some of the points
        // that are the closest to the center of the pixel first.
        .cycle()
        .skip(max_samples / 2)
        .take(max_samples)
    {
        let coloffset = (2.0 * f64::from(i) - ssaa_f64 - 1.0) / ssaa_f64;
        let rowoffset = (2.0 * f64::from(j) - ssaa_f64 - 1.0) / ssaa_f64;

        // Compute escape speed of point.
        // We use the potential instead of the number of
        // iterations in order to reduce color banding.
        let escape_speed = potential(
            pixel_region.center_real + rowoffset * pixel_region.real_distance,
            pixel_region.center_imag + coloffset * pixel_region.imag_distance,
            render_parameters.max_iterations,
        );

        // This branch will be the same for all iterations through the loop,
        // so the branch predictor should not have any issues with it.
        // This reasoning has been verified with benchmarks.
        color += match render_parameters.color_type {
            SupportedColorType::Rgb8 | SupportedColorType::Rgba8 => palette(escape_speed),
            SupportedColorType::L8 => LinearRGB::new(escape_speed, escape_speed, escape_speed),
        };

        samples += 1;

        // If we are far from the fractal we do not need to supersample.
        if RESTRICT_SSAA_REGION && escape_speed > SSAA_REGION_CUTOFF {
            if SHOW_SSAA_REGION {
                color = [150.0 / 255.0, 75.0 / 255.0, 0.0].into();
            }

            break;
        }
    }

    // Divide by the number of samples
    color /= f64::from(samples);
    // and convert to sRGB color space in the correct format.
    match render_parameters.color_type {
        SupportedColorType::L8 => Pixel::Luma(color.into()),
        SupportedColorType::Rgb8 => Pixel::Rgb(color.into()),
        SupportedColorType::Rgba8 => Pixel::Rgba(color.into()),
    }
}

/// Iterates the Mandelbrot function
///
/// ```math
/// z_(n+1) = z_n^2 + c
/// ```
///
/// on the given c starting with z_0 = c until it either escapes
/// or the loop exceeds the maximum number of iterations.
/// Returns a tuple of `(iterations, final |z|^2)`.
///
/// # Example
///
/// ```
/// # use mandellib::iterate;
/// # use core::num::NonZeroU32;
/// const MAXITERS: NonZeroU32 = NonZeroU32::new(10).unwrap();
/// // The origin is in the set
/// assert_eq!(iterate(0.0, 0.0, MAXITERS).0, MAXITERS.into());
///
/// // but 1 + i is not.
/// assert_ne!(iterate(1.0, 1.0, MAXITERS).0, MAXITERS.into());
///
/// // The magnitude of -2 never changes, regardless of iteration number.
/// assert_eq!(iterate(-2.0, 0.0, MAXITERS), (MAXITERS.into(), 4.0));
/// ```
///
/// # Note
///
/// Points inside the main cardioid or period-2 bulb are not iterated
/// but instead return immediately while reporting the maximum number of iterations.
/// For those points the modulus squared is not well defined and
/// is currently returned as NaN to indicate that the value should not be used.
///
/// ```
/// # use mandellib::iterate;
/// # use core::num::NonZeroU32;
/// # const MAXITERS: u32 = 100;
/// # let maxiters = NonZeroU32::new(MAXITERS).unwrap();
/// let (iters, broken_mag_sqr) = iterate(-1.0, 0.0, maxiters);
/// assert_eq!(iters, MAXITERS);
/// assert!(broken_mag_sqr.is_nan());
/// ```
#[must_use]
pub fn iterate(c_re: f64, c_im: f64, max_iterations: NonZeroU32) -> (u32, f64) {
    let c_imag_sqr = c_im * c_im;
    let mut mag_sqr = c_re * c_re + c_imag_sqr;

    let max_iterations = max_iterations.get();

    // Check whether the point is within the main cardioid or period 2 bulb.
    if CARDIOID_AND_BULB_CHECK && (c_re + 1.0) * (c_re + 1.0) + c_imag_sqr <= 0.0625
        || mag_sqr * (8.0 * mag_sqr - 3.0) <= 0.09375 - c_re
    {
        // We can unfortunately not know the final magnitude squared of the input in that case,
        // so we return that as NAN.
        return (max_iterations, f64::NAN);
    }

    let mut z_re = c_re;
    let mut z_im = c_im;
    let mut z_re_sqr = mag_sqr - c_imag_sqr;
    let mut z_im_sqr = c_imag_sqr;

    // We have effectively performed one iteration of the function
    // by setting the starting values as above.
    let mut iterations = 1;

    // Iterates the mandelbrot function.
    // This loop uses only 3 multiplications, which is the minimum.
    // While it is common to abort when |z| > 2 since such a point is guaranteed
    // to not be in the set, we keep iterating until |z| > 6 as this reduces
    // color banding.
    while iterations < max_iterations && mag_sqr <= 36.0 {
        z_im *= z_re;
        z_im += z_im;
        z_im += c_im;
        z_re = z_re_sqr - z_im_sqr + c_re;
        z_re_sqr = z_re * z_re;
        z_im_sqr = z_im * z_im;
        mag_sqr = z_re_sqr + z_im_sqr;
        iterations += 1;
    }

    (iterations, mag_sqr)
}

/// Returns a value kind of like the potential function of the Mandelbrot set.
/// Maps the result of [`iterate`] smoothly to a number between 0 (inside the set) and 1 (far outside).
#[must_use]
fn potential(c_re: f64, c_im: f64, max_iterations: NonZeroU32) -> f64 {
    let (iterations, mag_sqr) = iterate(c_re, c_im, max_iterations);

    let max_iterations = max_iterations.get();

    if iterations == max_iterations {
        // We label all points that could not be excluded as inside the set
        // This also avoids using the potentially undefined magnitude squared
        // for numbers that can be computed without iteration.
        0.0
    } else {
        // The shift of `e` is chosen becase it makes the final image look nicer with the current color curves.
        (f64::from(max_iterations - iterations) + mag_sqr.ln().log2() - std::f64::consts::E - 1.0)
            / f64::from(max_iterations)
    }
}

/// Contains information about a rectangle-shaped region in the complex plane.
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub center_real: f64,
    pub center_imag: f64,
    pub real_distance: f64,
    pub imag_distance: f64,
}

impl Frame {
    #[must_use]
    pub const fn new(
        center_real: f64,
        center_imag: f64,
        real_distance: f64,
        imag_distance: f64,
    ) -> Self {
        Self {
            center_real,
            center_imag,
            real_distance,
            imag_distance,
        }
    }
}

/// Contains information about the mandelbrot image
/// that is relevant to the rendering process.
#[derive(Debug, Clone, Copy)]
pub struct RenderParameters {
    pub x_resolution: U32AndUsize,
    pub y_resolution: U32AndUsize,
    pub max_iterations: NonZeroU32,
    pub sqrt_samples_per_pixel: NonZeroU8,
    pub color_type: SupportedColorType,
}

impl RenderParameters {
    /// # Errors
    /// Will return an error if `x_resolution` or `y_resolution` do not fit in a usize.
    pub fn try_new(
        x_resolution: NonZeroU32,
        y_resolution: NonZeroU32,
        max_iterations: NonZeroU32,
        sqrt_samples_per_pixel: NonZeroU8,
        color_type: SupportedColorType,
    ) -> Result<Self, TryFromIntError> {
        Ok(Self {
            x_resolution: x_resolution.try_into()?,
            y_resolution: y_resolution.try_into()?,
            max_iterations,
            sqrt_samples_per_pixel,
            color_type,
        })
    }
}

#[cfg(test)]
mod test_iteration {
    use super::*;

    #[test]
    fn check_some_iterations() {
        let max_iterations = NonZeroU32::new(255).unwrap();
        assert_eq!(iterate(0.0, 0.0, max_iterations).0, 255);
        assert_eq!(iterate(-2.0, 0.0, max_iterations).0, 255);
    }
}
