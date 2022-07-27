use clap::{Arg, Command};
use std::error::Error;
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
    pub fn new() -> Result<Config, Box<dyn Error>> {
        let mut center_real = "-0.75";
        let mut center_imag = "0.0";
        let mut aspect_ratio = "1.5";
        let imag_distance = 8.0 / 3.0;
        let mut resolution = "2160";
        let mut zoom = "1";
        let mut ssaa = "3";

        let matches = Command::new("mandelrust")
            .version("1.1.0")
            .author("Johanna Sörngård, jsorngard@gmail.com")
            .about("Renders an image of the Mandelbrot set")
            .arg(
                Arg::new("center_real")
                    .long("center-re")
                    .value_name("RE(CENTER)")
                    .help("the real part of the center point of the image")
                    .takes_value(true)
                    .required(false)
                    .allow_hyphen_values(true)
                    .default_value(center_real),
            )
            .arg(
                Arg::new("center_imag")
                    .long("center-im")
                    .value_name("IM(CENTER)")
                    .help("the imag part of the center point of the image")
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
                    .help("the aspect ratio of the image")
                    .takes_value(true)
                    .required(false)
                    .default_value(aspect_ratio),
            )
            .arg(
                Arg::new("resolution")
                    .short('n')
                    .long("number-of-points")
                    .value_name("RESOLUTION")
                    .help("the number of points along the imaginary axis to evaluate")
                    .takes_value(true)
                    .required(false)
                    .default_value(resolution),
            )
            .arg(
                Arg::new("no_save")
                    .short('x')
                    .help("do not write the results to file")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::new("zoom")
                    .short('z')
                    .long("zoom")
                    .value_name("ZOOM LEVEL")
                    .help("how far in to zoom on the given center point")
                    .takes_value(true)
                    .required(false)
                    .default_value(zoom),
            )
            .arg(
                Arg::new("ssaa")
                    .short('s')
                    .long("ssaa")
                    .value_name("SSAA")
                    .help("whether to supersample every pixel, and how much")
                    .takes_value(true)
                    .default_value(ssaa)
                    .required(false),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .help("print extra information")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        //Extract command line arguments
        center_real = matches.value_of("center_real").unwrap_or(center_real);
        center_imag = matches.value_of("center_imag").unwrap_or(center_imag);
        aspect_ratio = matches.value_of("aspect_ratio").unwrap_or(aspect_ratio);
        resolution = matches.value_of("resolution").unwrap_or(resolution);
        ssaa = matches.value_of("ssaa").unwrap_or(ssaa);
        let save_result = !matches.is_present("no_save");
        let verbose = matches.is_present("verbose");
        zoom = matches.value_of("zoom").unwrap_or(zoom);
        
        //Parse the inputs from strings into the appropriate types
        let center_real: f64 = center_real.trim().parse()?;
        let center_imag: f64 = center_imag.trim().parse()?;
        let aspect_ratio: f64 = aspect_ratio.trim().parse()?;
        let resolution: u32 = resolution.trim().parse()?;
        let ssaa: u32 = ssaa.trim().parse()?;
        let zoom: f64 = zoom.trim().parse()?;

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