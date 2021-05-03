use std::error::Error;

use clap::{App, Arg};
use image::RgbImage;

//Runs the main logic of the program and returns an error to
//main if something goes wrong
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let center_re = config.center_re;
    let center_im = config.center_im;
    let aspect_ratio = config.aspect_ratio;
    let yresolution = config.resolution;
    let save_result = config.save_result;
    let xresolution = (aspect_ratio * (yresolution as f64)) as u32;
    let zoom = config.zoom;
    println!(
        "The real part of the center point is {}, and the imaginary part is {}",
        center_re, center_im
    );
    println!(
        "The aspect ratio is {}, and the resolution is {}x{}",
        aspect_ratio, xresolution, yresolution
    );
    if save_result {
        println!("We should save the result");
    } else {
        println!("We should not save the result");
    }
    println!("We're gonna zoom in by a factor of {}", zoom);

    render(xresolution, yresolution, center_re, center_im, zoom);

    //Everything finished correctly!
    Ok(())
}

pub fn render(
    xresolution: u32,
    yresolution: u32,
    _center_re: f64,
    _center_im: f64,
    _zoom: f64,
) -> RgbImage {
    let _img_distance = 8.0 / 3.0;
    let mut img = RgbImage::new(xresolution, yresolution);
    for (x, y, _pixel) in img.enumerate_pixels_mut() {
        println!("x: {}, y: {}", x, y);
    }
    return img;
}

//Iterates the mandelbrot function on the input number until
//it either escapes or exceeds the maximum number of iterations
pub fn iterate(c_re: f64, c_im: f64, maxiterations: i64) -> f64 {
    let c_imag_sqr = c_im * c_im;
    let mag_sqr = c_re * c_re + c_imag_sqr;

    //Check whether the point is within the main cardioid or period 2 bulb
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

    //Iterates the mandelbrot function
    //This loop uses only 3 multiplications, which is the minimum
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

    (maxiterations - iterations) as f64
        - 4.0 * f64::powf((z_re_sqr + z_im_sqr).sqrt(), -0.4) / (maxiterations as f64)
}

//A struct containing the runtime specified configuration
//of the program
pub struct Config {
    pub center_re: f64,
    pub center_im: f64,
    pub aspect_ratio: f64,
    pub resolution: u32,
    pub save_result: bool,
    pub zoom: f64,
}

//Implementation of the Config struct
impl Config {
    //Returns a Result wrapper which contains a Config
    //struct if the arguments could be parsed correctly
    //and an error otherwise
    pub fn new() -> Result<Config, &'static str> {
        let mut center_re = "-0.75";
        let mut center_im = "0.0";
        let mut aspect_ratio = "1.5";
        let mut resolution = "2160";
        let mut save_result = true;
        let mut zoom = "1";

        let matches = App::new("rustybrot")
            .version("0.1")
            .author("Johanna Sörngård, jsorngard@gmail.com")
            .about("Renders an image of the Mandelbrot set")
            .arg(
                Arg::new("center_re")
                    .long("center-re")
                    .value_name("RE(CENTER)")
                    .about("the real part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .default_value(center_re),
            )
            .arg(
                Arg::new("center_im")
                    .long("center-im")
                    .value_name("IM(CENTER)")
                    .about("the imaginary part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .default_value(center_im),
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
            .get_matches();

        //Extract command line arguments
        if let Some(cr) = matches.value_of("center_re") {
            center_re = cr;
        }
        if let Some(ci) = matches.value_of("center_im") {
            center_im = ci;
        }
        if let Some(ar) = matches.value_of("aspect_ratio") {
            aspect_ratio = ar
        }
        if let Some(res) = matches.value_of("resolution") {
            resolution = res;
        }
        if matches.is_present("no_save") {
            save_result = false;
        }
        if let Some(z) = matches.value_of("zoom") {
            zoom = z;
        }

        //Parse the inputs from strings into the appropriate
        //types
        let center_re: f64 = match center_re.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret RE(CENTER) as a float"),
        };

        let center_im: f64 = match center_im.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret IM(CENTER) as a float"),
        };

        let aspect_ratio: f64 = match aspect_ratio.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret ASPECT RATIO as a float"),
        };

        let resolution: u32 = match resolution.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret RESOLUTION as an integer"),
        };

        let zoom: f64 = match zoom.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret ZOOM FACTOR as a float"),
        };

        Ok(Config {
            center_re,
            center_im,
            aspect_ratio,
            resolution,
            save_result,
            zoom,
        })
    }
}
