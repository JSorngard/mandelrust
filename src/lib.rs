use std::error::Error;

use clap::{App, Arg};
use image::RgbImage;

//Runs the main logic of the program and returns an error to
//main if something goes wrong
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
    println!(
        "The real part of the center point is {}, and the imag part is {}",
        center_real, center_imag
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

    let img = render(
        xresolution,
        yresolution,
        center_real,
        center_imag,
        imag_distance,
        real_distance,
        depth,
    );

    img.save("m.png").unwrap();

    //Everything finished correctly!
    Ok(())
}

pub fn render(
    xresolution: u32,
    yresolution: u32,
    center_real: f64,
    center_imag: f64,
    imag_distance: f64,
    real_distance: f64,
    depth: u8,
) -> RgbImage {
    let mut img = RgbImage::new(xresolution, yresolution);
    let start_real = center_real - real_distance / 2.0;
    let start_imag = center_imag - imag_distance / 2.0;
    let mut intensity;
    let mut c_real;
    let mut c_imag;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        c_real = start_real + real_distance * (x as f64) / (xresolution as f64);
        c_imag = start_imag + imag_distance * (y as f64) / (yresolution as f64);
        intensity = iterate(c_real, c_imag, depth as i64);
        *pixel = image::Rgb([
            (intensity * f64::powf(depth as f64, 1.0 - (intensity * 45.0) * 2.0)) as u8,
            (intensity * 70.0 - (880.0 * f64::powf(intensity, 18.0))
                + (701.0 * f64::powf(intensity, 9.0))) as u8,
            (intensity * 80.0 + (f64::powf(intensity, 9.0) * depth as f64)
                - (950.0 * f64::powf(intensity, 99.0))) as u8,
        ])
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
    pub center_real: f64,
    pub center_imag: f64,
    pub aspect_ratio: f64,
    pub imag_distance: f64,
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
        let mut center_real = "-0.75";
        let mut center_imag = "0.0";
        let mut aspect_ratio = "1.5";
        let imag_distance = 8.0 / 3.0;
        let mut resolution = "2160";
        let mut save_result = true;
        let mut zoom = "1";

        let matches = App::new("rustybrot")
            .version("0.1")
            .author("Johanna Sörngård, jsorngard@gmail.com")
            .about("Renders an image of the Mandelbrot set")
            .arg(
                Arg::new("center_real")
                    .long("center-re")
                    .value_name("RE(CENTER)")
                    .about("the real part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .default_value(center_real),
            )
            .arg(
                Arg::new("center_imag")
                    .long("center-im")
                    .value_name("IM(CENTER)")
                    .about("the imag part of the center point of the image")
                    .takes_value(true)
                    .required(false)
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
        if matches.is_present("no_save") {
            save_result = false;
        }
        if let Some(z) = matches.value_of("zoom") {
            zoom = z;
        }

        //Parse the inputs from strings into the appropriate
        //types
        let center_real: f64 = match center_real.trim().parse() {
            Ok(num) => num,
            Err(_) => return Err("Could not interpret RE(CENTER) as a float"),
        };

        let center_imag: f64 = match center_imag.trim().parse() {
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
            center_real,
            center_imag,
            aspect_ratio,
            imag_distance,
            resolution,
            save_result,
            zoom,
        })
    }
}
