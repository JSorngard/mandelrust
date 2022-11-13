use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize};
use std::error::Error;
use std::io::{stdout, Write};

use image::DynamicImage;
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};

// ----------- DEBUG FLAGS --------------
// Set to true to only super sample close to the border of the set.
const RESTRICT_SSAA_REGION: bool = true;

// Set to true to show the region where super sampling is skipped as brown.
// The region will have a black border where partial super sampling is done.
const SHOW_SSAA_REGION: bool = false;

// If the escape speed of a point is larger than this,
// supersampling will be aborted.
const SSAA_REGION_CUTOFF: f64 = 0.95;

// Set to false to never mirror the image
// regardless of resolution.
const ENABLE_MIRRORING: bool = true;
// --------------------------------------

const NUM_COLOR_CHANNELS: usize = 3;

/// Takes in variables describing where to render and at what resolution
/// and produces an image of the Mandelbrot set.
///
/// `render_parameters` contains `x_resolution`, `y_resolution`, `maxiterations`, `sqrt_samples_per_pixel` and `grayscale`.
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
/// `maxiterations` is the maximum number of iterations to compute for each pixel sample before labeling
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
    let mirror = ENABLE_MIRRORING
        && render_parameters.mirror
        && draw_region.center_imag.abs() < draw_region.imag_distance;

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
        // Split the image up into bands.
        .chunks_mut(NUM_COLOR_CHANNELS * y_resolution)
        .enumerate()
        // Iterate over the bands in parallel
        .par_bridge()
        // while showing a progress bar
        .progress_count(x_resolution.try_into()?)
        .for_each(|(x_index, band)| {
            // and color every pixel in each band
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

        // If we have rendered all the pixels with
        // negative imaginary part for this real part
        if mirror && c_imag > 0.0 {
            // we split the current pixel band up into two and
            let (mirror_src, mirror_dst) = band.split_at_mut(y_index);

            // `memcpy` the values of this pixel from one of the
            // already computed pixels.
            mirror_dst[0..NUM_COLOR_CHANNELS]
                .copy_from_slice(&mirror_src[(mirror_from - NUM_COLOR_CHANNELS)..mirror_from]);

            // Next loop iteration we want to mirror from the next pixel over.
            mirror_from -= NUM_COLOR_CHANNELS;
        } else {
            // Otherwise we compute the pixel color as normal by iteration.
            let escape_speed = supersampled_iterate(
                render_parameters.sqrt_samples_per_pixel,
                c_real,
                c_imag,
                real_delta,
                imag_delta,
                render_parameters.max_iterations,
            );

            let colors = if render_parameters.grayscale {
                [(f64::from(u8::MAX) * escape_speed) as u8; NUM_COLOR_CHANNELS]
            } else {
                map_escape_speed_to_color(escape_speed)
            };

            band[y_index..(NUM_COLOR_CHANNELS + y_index)].copy_from_slice(&colors);

            // We keep track of how many pixels have been colored
            // in order to potentially mirror them.
            mirror_from += NUM_COLOR_CHANNELS;
        }
    }
}

/// Determines the color of a pixel. The color map that this function uses was taken from the python code in
/// [this](https://preshing.com/20110926/high-resolution-mandelbrot-in-obfuscated-python/) blog post.
///
/// As the input increases from 0 to 1 the color transitions as
///
/// black -> brown -> orange -> yellow -> cyan -> blue -> dark blue -> black.
///
/// The function has not been tested for inputs outside the range \[0, 1\]
/// and makes no guarantees about the output in that case.
fn map_escape_speed_to_color(esc: f64) -> [u8; NUM_COLOR_CHANNELS] {
    let third_power = esc * esc * esc;
    let ninth_power = third_power * third_power * third_power;
    let eighteenth_power = ninth_power * ninth_power;
    let thirty_sixth_power = eighteenth_power * eighteenth_power;

    [
        (esc * 255.0_f64.powf(1.0 - 2.0 * ninth_power * thirty_sixth_power)) as u8,
        (esc * 70.0 - 880.0 * eighteenth_power + 701.0 * ninth_power) as u8,
        (esc * 80.0 + ninth_power * 255.0
            - 950.0 * thirty_sixth_power * thirty_sixth_power * eighteenth_power * ninth_power)
            as u8,
    ]
}

/// Computes the escape speed for the values in a grid
/// in a small region around the given value and returns their average.
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
pub fn supersampled_iterate(
    sqrt_samples_per_pixel: NonZeroU8,
    c_real: f64,
    c_imag: f64,
    real_delta: f64,
    imag_delta: f64,
    max_iterations: NonZeroU32,
) -> f64 {
    let ssaa = sqrt_samples_per_pixel.get();
    let f64ssaa: f64 = ssaa.into();

    //samples can be a u16 since the maximum number of samples is u8::MAX^2 which is less than u16::MAX
    let mut samples: u16 = 0;
    let max_samples: usize = (ssaa * ssaa).into();

    let mut escape_speed: f64 = 0.0;
    let mut coloffset: f64;
    let mut rowoffset: f64;
    let mut esc: f64;

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
        coloffset = (2.0 * f64::from(i) - f64ssaa - 1.0) / f64ssaa;
        rowoffset = (2.0 * f64::from(j) - f64ssaa - 1.0) / f64ssaa;

        // Compute escape speed of point.
        esc = iterate(
            c_real + rowoffset * real_delta,
            c_imag + coloffset * imag_delta,
            max_iterations,
        );
        escape_speed += esc;
        samples += 1;

        // If we are far from the fractal we do not need to supersample.
        if RESTRICT_SSAA_REGION && esc > SSAA_REGION_CUTOFF {
            if SHOW_SSAA_REGION {
                escape_speed = 0.5;
            }

            break;
        }
    }
    escape_speed /= f64::from(samples);
    escape_speed
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
        (f64::from(max_iterations - iterations) + (z_re_sqr + z_im_sqr).sqrt().ln().log2() - 2.8)
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
    pub mirror: bool,
}

impl RenderParameters {
    pub fn new(
        x_resolution: NonZeroUsize,
        y_resolution: NonZeroUsize,
        max_iterations: NonZeroU32,
        sqrt_samples_per_pixel: NonZeroU8,
        grayscale: bool,
        mirror: bool,
    ) -> Self {
        RenderParameters {
            x_resolution,
            y_resolution,
            max_iterations,
            sqrt_samples_per_pixel,
            grayscale,
            mirror,
        }
    }
}
