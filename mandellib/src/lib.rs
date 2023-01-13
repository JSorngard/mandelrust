use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize, TryFromIntError};
use std::io::{stdout, Write};

use image::{imageops, DynamicImage, ImageBuffer, Rgb};
use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use multiversion::multiversion;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    prelude::ParallelSliceMut,
};

use color_space::{palette, LinearRGB};

// ----------- DEBUG FLAGS --------------
// Set to true to only super sample close to the border of the set.
const RESTRICT_SSAA_REGION: bool = true;

// If the escape speed of a point is larger than this,
// supersampling will be aborted.
const SSAA_REGION_CUTOFF: f64 = 0.963;

// Set to true to display the region where supersampling is done
// as brown. The border region where supersampling is only partially done
// will appear as black.
const SHOW_SSAA_REGION: bool = false;

// Set to false to not mirror the image.
const ENABLE_MIRRORING: bool = true;
// --------------------------------------

const NUM_COLOR_CHANNELS: usize = 3;

/// Takes in variables describing where to render and at what resolution
/// and produces an image of the Mandelbrot set.
///
/// `render_parameters` contains `x_resolution`, `y_resolution`, `max_iterations`, `sqrt_samples_per_pixel` and `grayscale`.
///
/// `render_region` contains `center_real`, `centar_imag`, `real_distance` and `imag_distance`.
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
pub fn render(
    render_parameters: RenderParameters,
    render_region: Frame,
    verbose: bool,
) -> DynamicImage {
    let x_resolution = render_parameters.x_resolution;
    let y_resolution = render_parameters.y_resolution;

    let mut pixel_bytes: Vec<u8> =
        vec![0; NUM_COLOR_CHANNELS * x_resolution.usize.get() * y_resolution.usize.get()];

    let progress_bar = if verbose {
        ProgressBar::new(x_resolution.u32.get().into())
    } else {
        ProgressBar::hidden()
    };

    pixel_bytes
        // Split the image up into vertical bands and iterate over them in parallel.
        .par_chunks_exact_mut(NUM_COLOR_CHANNELS * y_resolution.usize.get())
        // We enumerate each band to be able to compute the real value of c for that band.
        .enumerate()
        .progress_with(progress_bar)
        .for_each(|(band_index, band)| {
            color_band(render_parameters, render_region, band_index, band)
        });

    if verbose {
        print!("\rProcessing image");
        if let Err(e) = stdout().flush() {
            eprintln!("unable to flush stdout (due to: {e}), continuing with rendering anyway");
        }
    }

    // The image is stored in a rotated fashion during rendering so that
    // the pixels of a column of the image lie contiguous in the backing vector.
    // Here we undo this rotation.
    let img = imageops::rotate270(
        &ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
            // This rotated state is the reason for the flipped image dimensions here.
            y_resolution.u32.get(),
            x_resolution.u32.get(),
            pixel_bytes,
        )
        .expect("`pixel_bytes` is allocated to the correct size of 3*xres*yres"),
    );

    if render_parameters.grayscale {
        DynamicImage::ImageLuma8(image::imageops::grayscale(&img))
    } else {
        DynamicImage::ImageRgb8(img)
    }
}

/// Computes the colors of the pixels in a y-axis band of the image of the mandelbrot set.
fn color_band(
    render_parameters: RenderParameters,
    render_region: Frame,
    band_index: usize,
    band: &mut [u8],
) {
    let x_resolution_f64 = f64::from(render_parameters.x_resolution.u32.get());
    let y_resolution_f64 = f64::from(render_parameters.y_resolution.u32.get());

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

    for y_index in (0..render_parameters.y_resolution.usize.get() * NUM_COLOR_CHANNELS)
        .step_by(NUM_COLOR_CHANNELS)
    {
        // Compute the imaginary part at this pixel
        let c_imag = start_imag
            + render_region.imag_distance * (y_index as f64)
                / (NUM_COLOR_CHANNELS as f64 * y_resolution_f64);

        if mirror && c_imag > 0.0 {
            // We have rendered every pixel with negative imaginary part.

            // We want to mirror from the next pixel over every iteration.
            // This line of code is before the mirroring since the first time
            // we enter this branch the pixel indicated by `mirror_from` is
            // the one that contains the real line, and we do not want to
            // mirror that one since the real line is infinitely thin.
            mirror_from -= NUM_COLOR_CHANNELS;

            // `memmove` the data from the already computed pixel into this one.
            band.copy_within((mirror_from - NUM_COLOR_CHANNELS)..mirror_from, y_index)
        } else {
            let pixel_region = Frame::new(c_real, c_imag, real_delta, imag_delta);

            // Otherwise we compute the pixel color as normal by iteration.
            let color = pixel_color(pixel_region, render_parameters);

            band[y_index..(NUM_COLOR_CHANNELS + y_index)].copy_from_slice(&color.0);

            // We keep track of how many pixels have been colored
            // in order to potentially mirror them.
            mirror_from += NUM_COLOR_CHANNELS;
        }
    }

    // If our assumption that we are rendering in the region of the complex plane with
    // negative imaginary component is false we must flip the vertical band
    // to get the correct image.
    if need_to_flip {
        for first_pixel_index in (0..band.len() / 2).step_by(NUM_COLOR_CHANNELS) {
            let opposite_pixel_index = band.len() - first_pixel_index - NUM_COLOR_CHANNELS;

            for channel_index in 0..NUM_COLOR_CHANNELS {
                band.swap(
                    first_pixel_index + channel_index,
                    opposite_pixel_index + channel_index,
                );
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
/// N.B.: if `sqrt_samples_per_pixel` is even the center of
/// the pixel is never sampled, and if it is 1 no super
/// sampling is done (only the center is sampled).
#[multiversion(targets = "simd")]
pub fn pixel_color(pixel_region: Frame, render_parameters: RenderParameters) -> Rgb<u8> {
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
        .skip(max_samples / 2)
        .cycle()
        .take(max_samples)
    {
        let coloffset = (2.0 * f64::from(i) - ssaa_f64 - 1.0) / ssaa_f64;
        let rowoffset = (2.0 * f64::from(j) - ssaa_f64 - 1.0) / ssaa_f64;

        // Compute escape speed of point.
        let escape_speed = iterate(
            pixel_region.center_real + rowoffset * pixel_region.real_distance,
            pixel_region.center_imag + coloffset * pixel_region.imag_distance,
            render_parameters.max_iterations,
        );

        color += if !render_parameters.grayscale {
            palette(escape_speed)
        } else {
            [escape_speed; 3].into()
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

    // Divide by the number of samples and convert to sRGB
    (color / f64::from(samples)).into()
}

/// Iterates the Mandelbrot function
///
/// z_(n+1) = z_n^2 + c
///
/// on the given c starting with z_0 = c until it either escapes
/// or the loop exceeds the maximum number of iterations.
/// Returns the escape speed of the point as a number between 0 and 1.
/// # Example
/// ```
/// # use mandellib::iterate;
/// # use core::num::NonZeroU32;
/// let maxiters = NonZeroU32::new(100).unwrap();
/// // The origin is in the set
/// assert_eq!(iterate(0.0, 0.0, maxiters), 0.0);
///
/// // and so is -2
/// assert_eq!(iterate(-2.0, 0.0, maxiters), 0.0);
///
/// // but 1 + i is not
/// assert_ne!(iterate(1.0, 1.0, maxiters), 0.0);
/// ```
#[multiversion(targets = "simd")]
pub fn iterate(c_re: f64, c_im: f64, max_iterations: NonZeroU32) -> f64 {
    let c_imag_sqr = c_im * c_im;
    let mag_sqr = c_re * c_re + c_imag_sqr;

    let max_iterations = max_iterations.get();

    // Check whether the point is within the main cardioid or period 2 bulb.
    if (c_re + 1.0) * (c_re + 1.0) + c_imag_sqr <= 0.0625
        || mag_sqr * (8.0 * mag_sqr - 3.0) <= 0.09375 - c_re
    {
        return 0.0;
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
    // to not be in the set, we keep iterating until |z| >= 6 as this reduces
    // color banding.
    while iterations < max_iterations && z_re_sqr + z_im_sqr <= 36.0 {
        z_im *= z_re;
        z_im += z_im;
        z_im += c_im;
        z_re = z_re_sqr - z_im_sqr + c_re;
        z_re_sqr = z_re * z_re;
        z_im_sqr = z_im * z_im;
        iterations += 1;
    }

    if iterations == max_iterations {
        0.0
    } else {
        // This takes the escape distance and the number of iterations to escape
        // and maps it smoothly to the range [0, 1] using the potential function to reduce color banding.
        // The shift of -2.8 is chosen for aesthetic reasons.
        (f64::from(max_iterations - iterations) - 2.8 + (z_re_sqr + z_im_sqr).ln().log2() - 1.0)
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
    pub fn new(center_real: f64, center_imag: f64, real_distance: f64, imag_distance: f64) -> Self {
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
    pub x_resolution: Resolution,
    pub y_resolution: Resolution,
    pub max_iterations: NonZeroU32,
    pub sqrt_samples_per_pixel: NonZeroU8,
    pub grayscale: bool,
}

impl RenderParameters {
    pub fn new(
        x_resolution: NonZeroU32,
        y_resolution: NonZeroU32,
        max_iterations: NonZeroU32,
        sqrt_samples_per_pixel: NonZeroU8,
        grayscale: bool,
    ) -> Result<Self, TryFromIntError> {
        Ok(Self {
            x_resolution: x_resolution.try_into()?,
            y_resolution: y_resolution.try_into()?,
            max_iterations,
            sqrt_samples_per_pixel,
            grayscale,
        })
    }
}

/// A struct containing a resolution that is known
/// to fit in both a u32 and usize type.
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub u32: NonZeroU32,
    pub usize: NonZeroUsize,
}

impl TryFrom<NonZeroU32> for Resolution {
    type Error = TryFromIntError;
    fn try_from(value: NonZeroU32) -> Result<Self, Self::Error> {
        Ok(Self {
            u32: value,
            usize: value.try_into()?,
        })
    }
}

#[cfg(test)]
mod test_iteration {
    use super::*;

    #[test]
    fn check_some_iterations() {
        let max_iterations = NonZeroU32::new(255).unwrap();
        assert_eq!(iterate(0.0, 0.0, max_iterations), 0.0);
        assert_eq!(iterate(-2.0, 0.0, max_iterations), 0.0);
    }
}
