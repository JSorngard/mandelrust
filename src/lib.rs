use std::error::Error;
use std::io::Write; //Needed for std::io::stdout() to exist in this scope

use clap::{App, Arg};
use image::RgbImage;

//Runs the main logic of the program and returns an error to
//main if something goes wrong.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let center_real = config.center_real;
    let center_imag = config.center_imag;
    let aspect_ratio = config.aspect_ratio;
    let yresolution = config.resolution;
    let save_result = config.save_result;
    let xresolution = (aspect_ratio * (yresolution as f64)) as u32;
    let zoom = config.zoom;
    let imag_distance = config.imag_distance / zoom;
    let real_distance = aspect_ratio * imag_distance;
    let depth = 255;
    let ssaa = config.ssaa;
    let real_delta = real_distance / (xresolution - 1) as f64;
    let imag_delta = imag_distance / (yresolution - 1) as f64;
    let verbose = config.verbose;

    //Output some basic information about what the program will be rendering.
    if verbose {
        print!("---- Generating a");
        if ssaa != 1 {
            print!(" {} times supersampled", u32::pow(ssaa, 2));
        } else {
            print!("n");
        }
        print!(
            " image with a resolution of {}x{}",
            xresolution, yresolution
        );
        if zoom != 1.0 {
            print!("zoomed by a factor of {}", zoom);
        }
        println!(" ----");
    }

    let img = render(
        xresolution,
        yresolution,
        ssaa,
        center_real,
        center_imag,
        real_delta,
        imag_delta,
        real_distance,
        imag_distance,
        depth,
        verbose,
    );

    if save_result {
        if verbose {
            print!("\rEncoding and saving image    ");
            flush();
        }
        img.save("m.png").unwrap();
    }
    if verbose {
        println!("\rDone                     ");
    }

    //Everything finished correctly!
    Ok(())
}

pub fn render(
    xresolution: u32,
    yresolution: u32,
    ssaa: u32,
    center_real: f64,
    center_imag: f64,
    real_delta: f64,
    imag_delta: f64,
    real_distance: f64,
    imag_distance: f64,
    depth: u8,
    verbose: bool,
) -> RgbImage {
    let invfactor: f64;
    if ssaa == 0 {
        invfactor = 0.0;
    } else {
        invfactor = 1.0 / (ssaa as f64);
    }

    let mirror = f64::abs(center_imag) < imag_distance; //True if the image contains the real axis, false otherwise.
    let mirror_sign: i32;
    //If the image contains the real axis we want to mirror
    //the result of the largest half on to the smallest.
    //One way of doing this is to always assume we are rendering
    //in lower half of the complex plane. If the assumption is false
    //we only need to flip the image vertically to get the
    //correct result since it is symmetric under conjugation.
    if center_imag >= 0.0 {
        mirror_sign = -1;
    } else {
        mirror_sign = 1;
    }
    let start_real = center_real - real_distance / 2.0;
    let start_imag = (mirror_sign as f64) * center_imag - imag_distance / 2.0;

    //We create a vector of u8's that will store the pixel information
    let mut pixels: Vec<u8> =
        Vec::with_capacity(xresolution as usize * yresolution as usize * 3 as usize);
    //Expand it to its full size so that we will not have to reallocate it again.
    for _i in 0..(xresolution * yresolution * 3) {
        pixels.push(0 as u8);
    }

    let mut c_real: f64;
    let mut previous_print: u32 = 0;
    let mut new_print: u32;
    let mut image_slice: &mut [u8];
    for x in 0..xresolution {
        c_real = start_real + real_distance * (x as f64) / (xresolution as f64);
        image_slice = &mut pixels[x as usize * yresolution as usize * 3 as usize
            ..yresolution as usize * (x as usize + 1 as usize) * 3 as usize];
        color_row(
            c_real,
            yresolution,
            start_imag,
            imag_distance,
            real_delta,
            imag_delta,
            mirror,
            ssaa,
            invfactor,
            depth,
            image_slice,
        );
        if verbose {
            new_print = 100 * x / xresolution;
            //Update progress only if we have something new to say.
            if new_print != previous_print {
                print!("\rComputing: {}%", new_print);
                flush();
                previous_print = new_print;
            }
        }
    }

    if verbose {
        print!("\rRendering image");
        flush();
    }
    let mut img =
        image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(yresolution, xresolution, pixels)
            .unwrap();

    if verbose {
        print!("\rProcessing image");
        flush();
    }
    img = image::imageops::rotate270(&img);
    if mirror_sign == -1 {
        img = image::imageops::flip_vertical(&img);
    }

    return img;
}

fn color_row(
    c_real: f64,
    yresolution: u32,
    start_imag: f64,
    imag_distance: f64,
    real_delta: f64,
    imag_delta: f64,
    mirror: bool,
    ssaa: u32,
    invfactor: f64,
    depth: u8,
    result: &mut [u8],
) {
    let mut c_imag: f64;
    let mut escape_speed: f64;
    let mut samples: u32;
    let mut coloffset: f64;
    let mut rowoffset: f64;
    let mut esc: f64;
    let mut mirror_from = 0;
    for y in (0..yresolution * 3).step_by(3) {
        c_imag = start_imag + imag_distance * (y as f64) / (3.0 * yresolution as f64);
        //If we have rendered all the pixels with
        //negative imaginary part for this real
        //part we mirror this pixel
        if mirror && c_imag > 0.0 {
            result[y as usize] = result[(mirror_from - 3) as usize];
            result[y as usize + 1 as usize] = result[mirror_from as usize - 2 as usize];
            result[y as usize + 2 as usize] = result[mirror_from as usize - 1 as usize];
            mirror_from -= 3;
        } else {
            //Reset supersampling variables.
            escape_speed = 0.0;
            samples = 0;

            //Supersampling loop.
            //Samples points in a grid around the intended point and averages
            //the results together to get a smoother image.
            for k in 1..=i64::pow(ssaa as i64, 2) {
                coloffset = ((k % (ssaa as i64) - 1) as f64) * invfactor;
                rowoffset = (((k - 1) as f64) / (ssaa as f64) - 1.0) * invfactor;

                //Compute escape speed of point.
                esc = iterate(
                    c_real + rowoffset * real_delta,
                    c_imag + coloffset * imag_delta,
                    depth as i64,
                );

                samples += 1;
                escape_speed += esc;

                //If we are far from the fractal we do not need to supersample.
                if esc > 0.9 {
                    //Uncomment the next line to only show supersampling region as non-black.
                    //escape_speed = 0.0;
                    break;
                }
            }
            escape_speed /= samples as f64;
            //Determine the color of the pixel. These color curves were found through experimentation.
            result[y as usize] = (escape_speed
                * f64::powf(depth as f64, 1.0 - f64::powf(escape_speed, 45.0) * 2.0))
                as u8;
            result[y as usize + 1 as usize] =
                (escape_speed * 70.0 - (880.0 * f64::powf(escape_speed, 18.0))
                    + (701.0 * f64::powf(escape_speed, 9.0))) as u8;
            result[y as usize + 2 as usize] =
                (escape_speed * 80.0 + (f64::powf(escape_speed, 9.0) * (depth as f64))
                    - (950.0 * f64::powf(escape_speed, 99.0))) as u8;
            mirror_from += 3;
        }
    }
}

//Flushes the stdout buffer.
fn flush() {
    std::io::stdout()
        .flush()
        .ok()
        .expect("could not flush stdout buffer");
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
    }

    if iterations == maxiterations {
        return 0.0;
    }

    ((maxiterations - iterations) as f64 - 4.0 * f64::powf((z_re_sqr + z_im_sqr).sqrt(), -0.4))
        / (maxiterations as f64)
}

//A struct containing the runtime specified configuration
//of the program.
pub struct Config {
    pub center_real: f64,
    pub center_imag: f64,
    pub aspect_ratio: f64,
    pub imag_distance: f64,
    pub resolution: u32,
    pub ssaa: u32,
    pub save_result: bool,
    pub zoom: f64,
    pub verbose: bool,
}

//Implementation of the Config struct.
impl Config {
    /*
    Returns a Result wrapper which contains a Config
    struct if the arguments could be parsed correctly
    and an error otherwise.
    */
    pub fn new() -> Result<Config, &'static str> {
        let mut center_real = "-0.75";
        let mut center_imag = "0.0";
        let mut aspect_ratio = "1.5";
        let imag_distance = 8.0 / 3.0;
        let mut resolution = "2160";
        let mut zoom = "1";
        let mut ssaa = "3";

        let matches = App::new("mandelrust")
            .version("1.1.0")
            .author("Johanna Sörngård, jsorngard@gmail.com")
            .about("Renders an image of the Mandelbrot set")
            .arg(
                Arg::new("center_real")
                    .long("center-re")
                    .value_name("RE(CENTER)")
                    .about("the real part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .allow_hyphen_values(true)
                    .default_value(center_real),
            )
            .arg(
                Arg::new("center_imag")
                    .long("center-im")
                    .value_name("IM(CENTER)")
                    .about("the imag part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .allow_hyphen_values(true)
                    .default_value(center_imag),
            )
            .arg(
                Arg::new("aspect_ratio")
                    .short('r')
                    .long("aspect-ratio")
                    .value_name("ASPECT RATIO")
                    .about("the aspect ratio of the image")
                    .takes_value(true)
                    .required(false)
                    .default_value(aspect_ratio),
            )
            .arg(
                Arg::new("resolution")
                    .short('n')
                    .long("number-of-points")
                    .value_name("RESOLUTION")
                    .about("the number of points along the imaginary axis to evaluate")
                    .takes_value(true)
                    .required(false)
                    .default_value(resolution),
            )
            .arg(
                Arg::new("no_save")
                    .short('x')
                    .about("do not write the results to file")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::new("zoom")
                    .short('z')
                    .long("zoom")
                    .value_name("ZOOM LEVEL")
                    .about("how far in to zoom on the given center point")
                    .takes_value(true)
                    .required(false)
                    .default_value(zoom),
            )
            .arg(
                Arg::new("ssaa")
                    .short('s')
                    .long("ssaa")
                    .value_name("SSAA")
                    .about("whether to supersample every pixel, and how much")
                    .takes_value(true)
                    .default_value(ssaa)
                    .required(false),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .about("print extra information")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        //Extract command line arguments
        if let Some(cr) = matches.value_of("center_real") {
            center_real = cr;
        }
        if let Some(ci) = matches.value_of("center_imag") {
            center_imag = ci;
        }
        if let Some(ar) = matches.value_of("aspect_ratio") {
            aspect_ratio = ar
        }
        if let Some(res) = matches.value_of("resolution") {
            resolution = res;
        }
        if let Some(s) = matches.value_of("ssaa") {
            ssaa = s;
        }
        let save_result = !matches.is_present("no_save");
        let verbose = matches.is_present("verbose");
        if let Some(z) = matches.value_of("zoom") {
            zoom = z;
        }

        //Parse the inputs from strings into the appropriate types
        let center_real: f64 = match center_real.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret RE(CENTER) as a float"),
        };

        let center_imag: f64 = match center_imag.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret IM(CENTER) as a float"),
        };

        let aspect_ratio: f64 = match aspect_ratio.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret ASPECT RATIO as a float"),
        };

        let resolution: u32 = match resolution.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret RESOLUTION as an integer"),
        };

        let ssaa: u32 = match ssaa.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret SSAA as an integer"),
        };

        let zoom: f64 = match zoom.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("could not interpret ZOOM FACTOR as a float"),
        };

        Ok(Config {
            center_real,
            center_imag,
            aspect_ratio,
            imag_distance,
            resolution,
            ssaa,
            save_result,
            zoom,
            verbose,
        })
    }
}
