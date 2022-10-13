use std::error::Error;
use std::io::{stdout, Write};
use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

/// Takes in variables describing where to render and at what resolution
/// and produces an image of the Mandelbrot set.
///
/// `render_parameters` contains `xresolution`, `yresolution`, `iterations`, `sqrt_samples_per_pixel` and `grayscale`.
///
/// `draw_region` contains `center_real`, `centar_imag`, `real_distance` and `imag_distance`.
///
/// `xresolution` and `yresolution` is the resolution in pixels in the real
/// and imaginary direction respectively.
/// `sqrt_samples_per_pixel` is the number of supersampled points along one direction. If it
/// is e.g. 3, then a supersampled pixel will be sampled 3^2 = 9 times.
/// region contains:
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
/// `xresolution` = `yresolution` = 100 and `center_real` = `center_imag` = 0 a square
/// of size 1x1 centered on the origin will be computed and rendered as a
/// 100x100 pixel image.
///
/// `iterations` is the maximum number of iterations to compute for each pixel sample before labeling
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
    let mirror = draw_region.center_imag.abs() < draw_region.imag_distance;

    // One way of doing this is to always assume we are rendering
    // in the lower half of the complex plane. If the assumption is false
    // we only need to flip the image vertically to get the
    // correct result since it is symmetric under conjugation.
    let mirror_sign = if draw_region.center_imag > 0.0 {
        -1.0
    } else {
        1.0
    };
    let start_real = draw_region.center_real - draw_region.real_distance / 2.0;
    let start_imag = mirror_sign * draw_region.center_imag - draw_region.imag_distance / 2.0;

    let xresolution = render_parameters.x_resolution;
    let yresolution = render_parameters.y_resolution;

    let pixel_bytes: Vec<u8> = vec![0; xresolution * yresolution * 3];
    let pixel_ptr = Arc::new(Mutex::new(pixel_bytes));

    // Make a parallel iterator over all the real values with rayon and for each
    (0..xresolution)
        .into_par_iter()
        .progress_count(xresolution.try_into().unwrap())
        .for_each(|real| {
            // compute the real part of c and
            let c_real =
                start_real + draw_region.real_distance * (real as f64) / (xresolution as f64);
            // color every pixel with that real value
            color_column(
                c_real,
                render_parameters,
                draw_region,
                real,
                start_imag,
                mirror,
                pixel_ptr.clone(),
            );
        });

    print!("\rRendering image");
    stdout().flush()?;

    // Extract the data from the mutex
    let finished_pixel_data =
        (*pixel_ptr.lock().expect("the mutex was poisoned, aborting")).clone();
    // and place it in an image buffer
    let mut img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(
        yresolution.try_into()?,
        xresolution.try_into()?,
        finished_pixel_data,
    )
    .ok_or("unable to construct image buffer from generated data")?;

    print!("\rProcessing image");
    stdout().flush()?;

    // Manipulate it to be the right side up and
    img = image::imageops::rotate270(&img);
    if mirror_sign == -1.0 {
        // flip it vertically if we need to due to mirroring
        image::imageops::flip_vertical_in_place(&mut img);
    }

    Ok(if render_parameters.grayscale {
        DynamicImage::ImageLuma8(image::imageops::grayscale(&img))
    } else {
        DynamicImage::ImageRgb8(img)
    })
}

/// Computes the colors of the pixels in a column of the image of the mandelbrot set.
fn color_column(
    c_real: f64,
    render_parameters: RenderParameters,
    draw_region: Frame,
    xindex: usize,
    start_imag: f64,
    mirror: bool,
    image: Arc<Mutex<Vec<u8>>>,
) {
    let xresolution = render_parameters.x_resolution;
    let yresolution = render_parameters.y_resolution;

    let grayscale = render_parameters.grayscale;
    let max_iterations = render_parameters.iterations;

    let mut mirror_from: usize = 0;
    let real_delta = draw_region.real_distance / (xresolution - 1) as f64;
    let imag_delta = draw_region.imag_distance / (yresolution - 1) as f64;

    // Create a temporary vector to hold the results for this row of pixels
    let mut result = vec![0; yresolution * 3];
    let mut c_imag: f64;
    for y in (0..yresolution * 3).step_by(3) {
        // Compute the imaginary part at this pixel
        c_imag = start_imag + draw_region.imag_distance * (y as f64) / (3.0 * yresolution as f64);

        // If we have rendered all the pixels with
        // negative imaginary part for this real
        // part we just mirror this pixel
        if mirror && c_imag > 0.0 {
            result[y] = result[mirror_from - 3];
            result[y + 1] = result[mirror_from - 2];
            result[y + 2] = result[mirror_from - 1];
            mirror_from -= 3;
        } else {
            let escape_speed = supersampled_iterate(
                render_parameters.sqrt_samples_per_pixel,
                c_real,
                c_imag,
                real_delta,
                imag_delta,
                max_iterations,
            );

            let colors = if grayscale {
                [(255.0 * escape_speed) as u8; 3]
            } else {
                map_luma_to_color(escape_speed)
            };

            result[y] = colors[0];
            result[y + 1] = colors[1];
            result[y + 2] = colors[2];
            mirror_from += 3;
        }
    }

    // Lock the mutex for the image pixels
    let mut pixels = image.lock().expect("mutex was poisoned, aborting");
    for (j, i) in (xindex * yresolution * 3..yresolution * (xindex + 1) * 3).enumerate() {
        // and copy the results into it
        pixels[i] = result[j];
    }
    // Unlock the mutex here by dropping the `MutexGuard` as it goes out of scope.
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
fn map_luma_to_color(luma: f64) -> [u8; 3] {
    [
        (luma * 255.0_f64.powf(1.0 - 2.0 * luma.powf(45.0))) as u8,
        (luma * 70.0 - (880.0 * luma.powf(18.0)) + (701.0 * luma.powf(9.0))) as u8,
        (luma * 80.0 + (luma.powf(9.0) * 255.0) - (950.0 * luma.powf(99.0))) as u8,
    ]
}

/// Computes the escape speed for the values in a small region
/// around the given value and returns their average.
/// If x is the location of `c_real` + `c_imag`*i and `sqrt_samples_per_pixel` = 3,
/// then the dots are also sampled:
/// ```text
///   real_delta
///    -------
///    .  .  .  |
///    .  x  .  | imag_delta
///    .  .  .  |
/// ```
pub fn supersampled_iterate(
    sqrt_samples_per_pixel: NonZeroU8,
    c_real: f64,
    c_imag: f64,
    real_delta: f64,
    imag_delta: f64,
    maxiterations: u32,
) -> f64 {
    let ssaa = sqrt_samples_per_pixel.get();

    let one_over_ssaa = if ssaa == 1 {
        0.0
    } else {
        1.0 / f64::from(ssaa)
    };

    let mut samples: u32 = 0;
    let mut escape_speed: f64 = 0.0;
    let mut coloffset: f64;
    let mut rowoffset: f64;
    let mut esc: f64;

    // Supersampling loop.
    for k in 1..i32::from(ssaa) * i32::from(ssaa) + 1 {
        coloffset = (f64::from(k % i32::from(ssaa) - 1)) * one_over_ssaa;
        rowoffset = (f64::from(k - 1) / f64::from(ssaa) - 1.0) * one_over_ssaa;

        // Compute escape speed of point.
        esc = iterate(
            c_real + rowoffset * real_delta,
            c_imag + coloffset * imag_delta,
            maxiterations,
        );
        escape_speed += esc;
        samples += 1;

        // If we are far from the fractal we do not need to supersample.
        if esc > 0.9 {
            // Uncomment the next line to only show supersampling region as non-black.
            // escape_speed = 0.0;
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
pub fn iterate(c_re: f64, c_im: f64, maxiterations: u32) -> f64 {
    let c_imag_sqr = c_im * c_im;
    let mag_sqr = c_re * c_re + c_imag_sqr;

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
    while iterations < maxiterations && z_re_sqr + z_im_sqr <= 36.0 {
        z_im *= z_re;
        z_im += z_im;
        z_im += c_im;
        z_re = z_re_sqr - z_im_sqr + c_re;
        z_re_sqr = z_re * z_re;
        z_im_sqr = z_im * z_im;
        iterations += 1;
    }

    if iterations == maxiterations {
        0.0
    } else {
        // This takes the escape distance, |z|, and the number of iterations to escape
        // and maps it smoothly to the range [0, 1]. This reduces color banding.
        (f64::from(maxiterations - iterations) - 4.0 * (z_re_sqr + z_im_sqr).sqrt().powf(-0.4))
            / f64::from(maxiterations)
        // -4*x^(-0.4) is an approximation of
        // ln(ln(x))/ln(2) - 2.8
        // that works well in the range 6 < x < 36, which is what's relevant for this implementation.
    }
}

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Copy, Debug)]
pub struct RenderParameters {
    pub x_resolution: usize,
    pub y_resolution: usize,
    pub iterations: u32,
    pub sqrt_samples_per_pixel: NonZeroU8,
    pub grayscale: bool,
}

impl RenderParameters {
    pub fn new(
        x_resolution: usize,
        y_resolution: usize,
        iterations: u32,
        sqrt_samples_per_pixel: NonZeroU8,
        grayscale: bool,
    ) -> Self {
        RenderParameters {
            x_resolution,
            y_resolution,
            iterations,
            sqrt_samples_per_pixel,
            grayscale,
        }
    }
}
