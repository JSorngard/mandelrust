use std::num::{NonZeroU32, NonZeroU8, NonZeroUsize, ParseFloatError};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Renders a supersampled image of the Mandelbrot set to a png file.
/// It is possible to change which part of the set is rendered, how zoomed in the image is,
/// the number of iterations to use, as well as a few other things.
pub struct Cli {
    // This struct contains the runtime specified configuration of the program.
    #[arg(
        short,
        long,
        value_name = "RE(CENTER)",
        default_value_t = -0.75,
        allow_hyphen_values = true,
    )]
    /// The real part of the center point of the image
    pub real_center: f64,

    #[arg(
        short,
        long,
        value_name = "IM(CENTER)",
        default_value_t = 0.0,
        allow_hyphen_values = true
    )]
    /// The imaginary part of the center point of the image
    pub imag_center: f64,

    #[arg(
        short,
        long,
        value_parser(positive_double),
        value_name = "ZOOM LEVEL",
        default_value_t = 1.0
    )]
    /// How far in to zoom on the given center point. If this is 2 the image is zoomed by a factor of 2,
    /// meaning the vertical and horizontal distance that the image covers in the complex plane is halved.
    /// In general these distances scale as 1/zoom
    pub zoom: f64,

    #[arg(
        short,
        long,
        // unwrap is okay because 2160 is not 0.
        default_value_t = NonZeroUsize::new(2160).unwrap(),
    )]
    /// The number of pixels along the y-axis of the image
    pub pixels: NonZeroUsize,

    #[arg(short, long, value_parser(positive_double), default_value_t = 1.5)]
    /// The aspect ratio of the image. The horizontal pixel resolution is calculated by multiplying the
    /// vertical pixel resolution by this number
    pub aspect_ratio: f64,

    #[arg(
        short,
        long,
        value_name = "SQRT(SSAA FACTOR)",
        // unwrap is okay because 4 is not 0.
        default_value_t = NonZeroU8::new(4).unwrap(),
    )]
    /// How many samples to compute for each pixel (along one direction, the actual number of samples is the square of this number).
    /// If this is set to 1, supersampling is turned off
    pub ssaa: NonZeroU8,

    #[arg(
        short,
        long,
        value_name = "MAX ITERATIONS",
        // unwrap is okay because 255 is not 0
        default_value_t = NonZeroU32::new(255).unwrap(),
    )]
    /// The maximum number of iterations for each pixel sample
    pub max_iterations: NonZeroU32,

    #[arg(long)]
    /// Output the image in grayscale by linearly mapping the escape speed of each pixel to a luma value between 0 and 255
    pub grayscale: bool,

    #[arg(long)]
    /// Save information about the image location in the complex plane in the file name
    pub record_params: bool,

    #[arg(short, long, default_value = "renders", value_name = "OUTPUT FOLDER")]
    /// The folder in which to save the resulting image
    pub output_folder: String,

    #[arg(long)]
    /// Do not mirror the image in the real axis
    pub disable_mirroring: bool,
}

/// Tries to parse the input string slice into an f64 > 0.
fn positive_double(s: &str) -> Result<f64, String> {
    let x: f64 = s.parse().map_err(|e: ParseFloatError| e.to_string())?;

    if x > 0.0 {
        Ok(x)
    } else {
        Err("the value must be positive".into())
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}