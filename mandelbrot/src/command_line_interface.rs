use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize};

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
        value_name = "ZOOM LEVEL",
        default_value_t = 0.0,
        allow_hyphen_values = true
    )]
    /// A real number describing how far in to zoom on the given center point.
    /// This number works on an exponential scale where 0 means no zoom
    /// and every time it is increased by 1 the vertical and horizontal
    /// distances covered by the image are halved
    pub zoom: f64,

    #[arg(
        short,
        long,
        // unwrap is okay because 2160 is not 0.
        default_value_t = NonZeroUsize::new(2160).unwrap(),
    )]
    /// The number of pixels along the y-axis of the image
    pub pixels: NonZeroUsize,

    #[arg(
        short,
        long,
        value_name = "ASPECT RATIO",
        value_parser(parse_aspect_ratio),
        default_value_t = 1.5
    )]
    /// The aspect ratio of the image. The horizontal pixel resolution is calculated by multiplying the
    /// vertical pixel resolution by this number. The aspect ratio can be entered as a real number and also in the format x:y,
    /// where x and y are integers, e.g. 3:2
    pub aspect_ratio: f64,

    #[arg(
        short,
        long,
        value_name = "SQRT(SSAA FACTOR)",
        // unwrap is okay because 3 is not 0.
        default_value_t = NonZeroU8::new(3).unwrap(),
    )]
    /// How many samples to compute for each pixel along one dimension. The total number of samples per pixel is the square of this number.
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

    #[arg(
        short,
        long,
        default_value = "mandelbrot_renders",
        value_name = "OUTPUT FOLDER"
    )]
    /// The folder in which to save the resulting image
    pub output_folder: String,

    #[arg(short, long)]
    /// Print extra information and show the progress of the rendering process
    pub verbose: bool,
}

/// Tries to interpret the input string as if it is an aspect ratio.
/// 3:2 and 1.5 both work.
fn parse_aspect_ratio(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(x_by_y) => {
            if x_by_y > 0.0 {
                Ok(x_by_y)
            } else {
                Err("aspect ratio must be larger than zero".into())
            }
        }
        Err(_) => {
            let substrings: Vec<&str> = s.split(':').collect();
            if substrings.len() == 2 {
                match (
                    substrings[0].parse::<NonZeroU32>(),
                    substrings[1].parse::<NonZeroU32>(),
                ) {
                    (Ok(x), Ok(y)) => Ok(f64::from(x.get()) / f64::from(y.get())),
                    (Ok(_), Err(e)) | (Err(e), Ok(_)) => Err(e.to_string()),
                    (Err(e1), Err(e2)) => Err(format!(
                        "horizontal integer has issue: '{}' vertical integer has issue '{}'",
                        e1, e2,
                    )),
                }
            } else {
                Err("input could not be interpreted as an aspect ratio".into())
            }
        }
    }
}

#[cfg(test)]
mod test_cli {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}