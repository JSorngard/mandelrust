use std::io::{Write, stdout};
use std::sync::{Mutex, Arc};
use std::error::Error;

use image::RgbImage;
use rayon::prelude::*;
use indicatif::ParallelProgressIterator;

///Takes in variables describing where to render and at what resolution
///and produces an image of the Mandelbrot set.
///xresolution and yresolution is the resolution in pixels in the real
///and imaginary direction respectively.
///ssaa is the number of supersampled points along one direction. If ssaa
///is e.g. 3, then a supersampled pixel will be sampled 3^2 = 9 times.
///center_real and center_imag are the real and imaginary parts of the
///point at the center of the image.
///real_distance and imag_distance describe the size of the region in the
///complex plane to render. E.g. if real_distance = imag_distance = 1,
///xresolution = yresolution = 100 and center = 0+0i a square of size 1x1
///centered on the origin will be computed and rendered as a 100x100 pixel
///image.
///If verbose is true the function will print progress information to stdout.
pub fn render(
    xresolution: u32,
    yresolution: u32,
    ssaa: u32,
    center_real: f64,
    center_imag: f64,
    real_distance: f64,
    imag_distance: f64,
    verbose: bool,
) -> Result<RgbImage, Box<dyn Error>> {

    //True if the image contains the real axis, false otherwise.
    //If the image contains the real axis we want to mirror
    //the result of the largest half on to the smallest.
    //One way of doing this is to always assume we are rendering
    //in lower half of the complex plane. If the assumption is false
    //we only need to flip the image vertically to get the
    //correct result since it is symmetric under conjugation.
    let mirror = f64::abs(center_imag) < imag_distance;

    let mirror_sign = if center_imag >= 0.0 { -1 } else { 1 };
    let start_real = center_real - real_distance / 2.0;
    let start_imag = (mirror_sign as f64) * center_imag - imag_distance / 2.0;

    let pixel_bytes: Vec<u8> = vec![0; xresolution as usize * yresolution as usize * 3];
    let pixel_ptr = Arc::new(Mutex::new(pixel_bytes));



    (0..xresolution).into_par_iter().progress_count(xresolution.into()).map(|real| {
        //Compute the real part of c.
        let c_real = start_real + real_distance * (real as f64) / (xresolution as f64);
        color_column(
            c_real,
            xresolution,
            yresolution,
            real as usize,
            real_distance,
            imag_distance,
            start_imag,
            mirror,
            ssaa,
            pixel_ptr.clone(),
        );
        real
    }).for_each(|_| ());

    if verbose {
        print!("\rRendering image");
        stdout().flush()?;
    }
    let finished_pixel_data = pixel_ptr.lock().unwrap();
    let mut img =
        image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(yresolution, xresolution, (*finished_pixel_data).clone()).unwrap();

    if verbose {
        print!("\rProcessing image");
        stdout().flush()?;
    }

    img = image::imageops::rotate270(&img);
    if mirror_sign == -1 {
        img = image::imageops::flip_vertical(&img);
    }

    Ok(img)
}

///Computes the colors of the pixels in a column of the image of the mandelbrot set.
fn color_column(
    c_real: f64,
    xresolution: u32,
    yresolution: u32,
    xindex: usize,
    real_distance: f64,
    imag_distance: f64,
    start_imag: f64,
    mirror: bool,
    ssaa: u32,
    image: Arc<Mutex<Vec<u8>>>,
) {
    let mut c_imag: f64;
    let mut mirror_from = 0;
    let depth: u64 = 255;
    let real_delta = real_distance / (xresolution - 1) as f64;
    let imag_delta = imag_distance / (yresolution - 1) as f64;

    //Create a temporary vector to hold the results for this row of pixels
    let mut result = vec![0; usize::try_from(yresolution*3).unwrap()];

    for y in (0..yresolution * 3).step_by(3) {
        c_imag = start_imag + imag_distance * (y as f64) / (3.0 * yresolution as f64);
        //If we have rendered all the pixels with
        //negative imaginary part for this real
        //part we just mirror this pixel
        if mirror && c_imag > 0.0 {
            result[y as usize] = result[(mirror_from - 3) as usize];
            result[y as usize + 1] = result[mirror_from as usize - 2];
            result[y as usize + 2] = result[mirror_from as usize - 1];
            mirror_from -= 3;
        } else {
            let colors = color_pixel(
                supersampled_iterate(ssaa, c_real, c_imag, real_delta, imag_delta, depth),
                depth,
            );
            result[y as usize] = colors[0];
            result[y as usize + 1] = colors[1];
            result[y as usize + 2] = colors[2];
            mirror_from += 3;
        }
    }

    //Unlock the mutex for the image pixels
    let mut pixels = image.lock().unwrap();
    for (j, i) in (xindex * yresolution as usize * 3..yresolution as usize * (xindex + 1) * 3).enumerate() {
        //and copy the results into it
        pixels[i] = result[j];
    }
}

///Determines the color of a pixel. These color curves were found through experimentation.
fn color_pixel(escape_speed: f64, depth: u64) -> [u8; 3] {
    [
        (escape_speed * f64::powf(depth as f64, 1.0 - f64::powf(escape_speed, 45.0) * 2.0)) as u8,
        (escape_speed * 70.0 - (880.0 * f64::powf(escape_speed, 18.0))
            + (701.0 * f64::powf(escape_speed, 9.0))) as u8,
        (escape_speed * 80.0 + (f64::powf(escape_speed, 9.0) * (depth as f64))
            - (950.0 * f64::powf(escape_speed, 99.0))) as u8,
    ]
}

fn supersampled_iterate(
    ssaa: u32,
    c_real: f64,
    c_imag: f64,
    real_delta: f64,
    imag_delta: f64,
    depth: u64,
) -> f64 {
    let one_over_ssaa = if ssaa == 0 { 0.0 } else { 1.0 / (ssaa as f64) };

    let mut samples: u32 = 0;
    let mut escape_speed: f64 = 0.0;
    let mut coloffset: f64;
    let mut rowoffset: f64;
    let mut esc: f64;

    //Supersampling loop.
    //Samples points in a grid around the intended point and averages
    //the results together to get a smoother image.
    for k in 1..=i64::pow(ssaa as i64, 2) {
        coloffset = ((k % (ssaa as i64) - 1) as f64) * one_over_ssaa;
        rowoffset = (((k - 1) as f64) / (ssaa as f64) - 1.0) * one_over_ssaa;

        //Compute escape speed of point.
        esc = iterate(
            c_real + rowoffset * real_delta,
            c_imag + coloffset * imag_delta,
            depth as i64,
        );
        escape_speed += esc;
        samples += 1;

        //If we are far from the fractal we do not need to supersample.
        if esc > 0.9 {
            //Uncomment the next line to only show supersampling region as non-black.
            //escape_speed = 0.0;
            break;
        }
    }
    escape_speed /= samples as f64;
    escape_speed
}

/*
Iterates the mandelbrot function on the input number until
it either escapes or exceeds the maximum number of iterations.
*/
pub fn iterate(c_re: f64, c_im: f64, maxiterations: i64) -> f64 {
    let c_imag_sqr = c_im * c_im;
    let mag_sqr = c_re * c_re + c_imag_sqr;

    //Check whether the point is within the main cardioid or period 2 bulb.
    if f64::powf(c_re + 1.0, 2.0) + c_imag_sqr <= 0.0625
        || mag_sqr * (8.0 * mag_sqr - 3.0) <= 0.09375 - c_re
    {
        return 0.0;
    }

    let mut z_re = 0.0;
    let mut z_im = 0.0;
    let mut z_re_sqr = 0.0;
    let mut z_im_sqr = 0.0;
    let mut iterations = 0;
    let mut old_re = 0.0;
    let mut old_im = 0.0;
    let mut period = 0;
    let tol = 1e-8;

    //Iterates the mandelbrot function.
    //This loop uses only 3 multiplications, which is the minimum.
    while iterations < maxiterations && z_re_sqr + z_im_sqr <= 36.0 {
        z_im *= z_re;
        z_im += z_im;
        z_im += c_im;
        z_re = z_re_sqr - z_im_sqr + c_re;
        z_re_sqr = z_re * z_re;
        z_im_sqr = z_im * z_im;
        iterations += 1;

        if f64::abs(z_re - old_re) < tol && f64::abs(z_im - old_im) < tol {
            return 0.0;
        }

        period += 1;
        if period > 10 {
            period = 0;
            old_re = z_re;
            old_im = z_im;
        }
    }

    if iterations == maxiterations {
        return 0.0;
    }

    ((maxiterations - iterations) as f64 - 4.0 * f64::powf((z_re_sqr + z_im_sqr).sqrt(), -0.4))
        / (maxiterations as f64)
}
