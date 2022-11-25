use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize};
use std::error::Error;
use std::io::{stdout, Write};

use image::{DynamicImage, Rgb};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    prelude::ParallelSliceMut,
};

use crate::color_space::LinearRGB;

// ----------- DEBUG FLAGS --------------
// Set to true to only super sample close to the border of the set.
const RESTRICT_SSAA_REGION: bool = true;

// If the escape speed of a point is larger than this,
// supersampling will be aborted.
const SSAA_REGION_CUTOFF: f64 = 0.963;

// Set to true to dsiplay the region where supersampling is done
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
/// `draw_region` contains `center_real`, `centar_imag`, `real_distance` and `imag_distance`.
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
pub fn render(
    render_parameters: RenderParameters,
    draw_region: Frame,
) -> Result<DynamicImage, Box<dyn Error>> {
    // True if the image contains the real axis, false otherwise.
    // If the image contains the real axis we want to mirror
    // the result of the largest half on to the smallest.
    let mirror = ENABLE_MIRRORING && draw_region.center_imag.abs() < draw_region.imag_distance;

    // One way of doing this is to always assume that the half with negative
    // imaginary part is the larger one. If the assumption is false
    // we only need to flip the image vertically to get the
    // correct result since it is symmetric under conjugation.
    let need_to_flip = draw_region.center_imag > 0.0;
    let start_real = draw_region.center_real - draw_region.real_distance / 2.0;
    let start_imag = if need_to_flip { -1.0 } else { 1.0 } * draw_region.center_imag
        - draw_region.imag_distance / 2.0;

    let x_resolution = render_parameters.x_resolution.get();
    let y_resolution = render_parameters.y_resolution.get();

    let mut pixels: Vec<u8> = vec![0; NUM_COLOR_CHANNELS * x_resolution * y_resolution];

    pixels
        // Split the image up into vertical bands and iterate over them in parallel.
        .par_chunks_mut(NUM_COLOR_CHANNELS * y_resolution)
        // We enumerate each band to be able to compute the real value of c for that band.
        .enumerate()
        .progress_count(x_resolution.try_into()?)
        .for_each(|(x_index, band)| {
            color_band(
                start_real + draw_region.real_distance * (x_index as f64) / (x_resolution as f64),
                render_parameters,
                draw_region,
                start_imag,
                mirror,
                band,
            )
        });

    // Place the data in an image buffer
    let mut img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(
        // The image is stored in a transposed fashion so that the pixels
        // of a column of the image lie contiguous in the backing vector.
        y_resolution.try_into()?,
        x_resolution.try_into()?,
        pixels,
    )
    .ok_or("unable to construct image buffer from generated data")?;

    print!("\rProcessing image");
    stdout().flush()?;

    // Undo the transposed state used during rendering and
    img = image::imageops::rotate270(&img);
    if need_to_flip {
        // flip the image vertically if we need to due to mirroring
        image::imageops::flip_vertical_in_place(&mut img);
    }

    if render_parameters.grayscale {
        Ok(DynamicImage::ImageLuma8(image::imageops::grayscale(&img)))
    } else {
        Ok(DynamicImage::ImageRgb8(img))
    }
}

/// Computes the colors of the pixels in a y-axis band of the image of the mandelbrot set.
fn color_band(
    c_real: f64,
    render_parameters: RenderParameters,
    draw_region: Frame,
    start_imag: f64,
    mirror: bool,
    band: &mut [u8],
) {
    let y_resolution = render_parameters.y_resolution.get();

    let mut mirror_from: usize = 0;
    let real_delta = draw_region.real_distance / (render_parameters.x_resolution.get() - 1) as f64;
    let imag_delta = draw_region.imag_distance / (y_resolution - 1) as f64;

    for y_index in (0..y_resolution * NUM_COLOR_CHANNELS).step_by(NUM_COLOR_CHANNELS) {
        // Compute the imaginary part at this pixel
        let c_imag = start_imag
            + draw_region.imag_distance * (y_index as f64)
                / (NUM_COLOR_CHANNELS as f64 * y_resolution as f64);

        if mirror && c_imag > 0.0 {
            // We have rendered every pixel with negative imaginary part.

            // We want to mirror from the next pixel over every iteration.
            // This line of code is before the mirroring since the first time
            // we enter this branch the pixel indicated by `mirror_from` is
            // the one that contains the real line, and we do not want to
            // mirror that one since the real line is infinitely thin.
            mirror_from -= NUM_COLOR_CHANNELS;

            let (mirror_src, mirror_dst) = band.split_at_mut(y_index);

            // `memcpy` the values of this pixel from one of the
            // already computed pixels.
            mirror_dst[0..NUM_COLOR_CHANNELS]
                .copy_from_slice(&mirror_src[(mirror_from - NUM_COLOR_CHANNELS)..mirror_from]);
        } else {
            // Otherwise we compute the pixel color as normal by iteration.
            let color = supersampled_pixel_color(
                render_parameters.sqrt_samples_per_pixel,
                c_real,
                c_imag,
                real_delta,
                imag_delta,
                render_parameters,
            );

            band[y_index..(NUM_COLOR_CHANNELS + y_index)].copy_from_slice(&color.0);

            // We keep track of how many pixels have been colored
            // in order to potentially mirror them.
            mirror_from += NUM_COLOR_CHANNELS;
        }
    }
}

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
fn palette(escape_speed: f64) -> LinearRGB {
    let third_power = escape_speed * escape_speed * escape_speed;
    let ninth_power = third_power * third_power * third_power;
    let eighteenth_power = ninth_power * ninth_power;
    let thirty_sixth_power = eighteenth_power * eighteenth_power;

    LinearRGB::from(Rgb::from([
        255.0_f64.powf(-2.0 * ninth_power * thirty_sixth_power) * escape_speed,
        14.0 / 51.0 * escape_speed - 176.0 / 51.0 * eighteenth_power + 701.0 / 255.0 * ninth_power,
        16.0 / 51.0 * escape_speed + ninth_power
            - 190.0 / 51.0
                * thirty_sixth_power
                * thirty_sixth_power
                * eighteenth_power
                * ninth_power,
    ]))
}

/// Computes the escape speed for the values in a grid
/// in a small region around the given value, computes their resulting
/// colors and returns the average color as an sRGB value.
/// If x is the location of `c_real` + `c_imag`*i and
/// `sqrt_samples_per_pixel` = 3, then the dots are also sampled:
///
/// ```text
///   real_delta
///    -------
///    .  .  .  |
///    .  x  .  | imag_delta
///    .  .  .  |
/// ```
///
/// The gap between the sample points at the edge and the
/// edge of the pixel is the same as between the points.
///
/// N.B.: if `sqrt_samples_per_pixel` is even, the center of
/// the pixel is never sampled.
pub fn supersampled_pixel_color(
    sqrt_samples_per_pixel: NonZeroU8,
    c_real: f64,
    c_imag: f64,
    real_delta: f64,
    imag_delta: f64,
    render_parameters: RenderParameters,
) -> Rgb<u8> {
    let ssaa = sqrt_samples_per_pixel.get();
    let f64ssaa: f64 = ssaa.into();

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
        let coloffset = (2.0 * f64::from(i) - f64ssaa - 1.0) / f64ssaa;
        let rowoffset = (2.0 * f64::from(j) - f64ssaa - 1.0) / f64ssaa;

        // Compute escape speed of point.
        let escape_speed = iterate(
            c_real + rowoffset * real_delta,
            c_imag + coloffset * imag_delta,
            render_parameters.max_iterations,
        );

        let color_sample = if !render_parameters.grayscale {
            palette(escape_speed)
        } else {
            [escape_speed; 3].into()
        };

        color += color_sample;
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
        // This takes the escape distance, |z|, and the number of iterations to escape
        // and maps it smoothly to the range [0, 1]. This reduces color banding.
        (f64::from(max_iterations - iterations) + (z_re_sqr + z_im_sqr).ln().log2() - 3.8)
            / f64::from(max_iterations)
    }
}

/// Contains information about a rectangle-shaped region in the complex plane.
#[derive(Clone, Copy)]
pub struct Frame {
    pub center_real: f64,
    pub center_imag: f64,
    pub real_distance: f64,
    pub imag_distance: f64,
}

impl Frame {
    pub fn new(center_real: f64, center_imag: f64, real_distance: f64, imag_distance: f64) -> Self {
        Frame {
            center_real,
            center_imag,
            real_distance,
            imag_distance,
        }
    }
}

/// Contains information about the mandelbrot image
/// that is relevant to the rendering process.
#[derive(Clone, Copy)]
pub struct RenderParameters {
    pub x_resolution: NonZeroUsize,
    pub y_resolution: NonZeroUsize,
    pub max_iterations: NonZeroU32,
    pub sqrt_samples_per_pixel: NonZeroU8,
    pub grayscale: bool,
}

impl RenderParameters {
    pub fn new(
        x_resolution: NonZeroUsize,
        y_resolution: NonZeroUsize,
        max_iterations: NonZeroU32,
        sqrt_samples_per_pixel: NonZeroU8,
        grayscale: bool,
    ) -> Self {
        RenderParameters {
            x_resolution,
            y_resolution,
            max_iterations,
            sqrt_samples_per_pixel,
            grayscale,
        }
    }
}
