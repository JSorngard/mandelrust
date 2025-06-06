use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize};

use clap::Parser;

use crate::resolution::Resolution;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Renders a supersampled image of the Mandelbrot set to an image file.
/// It is possible to change which part of the set is rendered, how zoomed in the image is,
/// the number of iterations to use, as well as a few other things.
pub struct Cli {
    // This struct contains the runtime specified configuration of the program.
    #[arg(
        short,
        long,
        value_name = "RE(CENTER)",
        allow_negative_numbers = true,
        default_value_t = -0.75
    )]
    /// The real part of the center point of the image
    pub real_center: f64,

    #[arg(
        short,
        long,
        value_name = "IM(CENTER)",
        allow_negative_numbers = true,
        default_value_t = 0.0
    )]
    /// The imaginary part of the center point of the image
    pub imag_center: f64,

    #[arg(short, long, default_value_t = 0.0, allow_negative_numbers = true)]
    /// A real number describing how far in to zoom on the given center point.
    /// This number works on an exponential scale where 0 means no zoom
    /// and every time it is increased by 1 the vertical and horizontal
    /// distances covered by the image are halved
    pub zoom_level: f64,

    #[arg(
        short = 'p',
        value_name = "X_RESxY_RES",
        long,
        default_value_t = const {Resolution::new(3240, 2160).expect("3240 and 2160 are not 0")},
    )]
    /// The resolution of the image in the form "X_RESxY_RES", e.g. "3240x2160"
    pub resolution: Resolution,

    #[arg(
        short,
        long,
        value_name = "SQRT(SSAA_FACTOR)",
        default_value_t = const {NonZeroU8::new(3).expect("3 is not 0")},
    )]
    /// How many samples to compute for each pixel along one dimension.
    /// The total number of samples per pixel is the square of this number.
    /// If this is set to 1, supersampling is turned off
    pub ssaa: NonZeroU8,

    #[arg(
        short,
        long,
        default_value_t = const {NonZeroU32::new(255).expect("255 is not 0")},
    )]
    /// The maximum number of iterations for each pixel sample
    pub max_iterations: NonZeroU32,

    #[arg(long)]
    /// Output the image in grayscale by mapping escape speed to brightness
    pub grayscale: bool,

    #[arg(short, long, default_value_t = String::from("mandelbrot_set.png"))]
    /// The path at which to save the resulting image.
    /// Supports saving as png
    #[cfg_attr(feature = "jpg", doc = ", jpg")]
    #[cfg_attr(feature = "webp", doc = ", webp")]
    #[cfg_attr(feature = "tiff", doc = ", tiff")]
    #[cfg_attr(feature = "bmp", doc = ", bmp")]
    #[cfg_attr(feature = "qoi", doc = ", qoi")]
    #[cfg_attr(feature = "gif", doc = ", gif")]
    #[cfg_attr(feature = "ico", doc = ", ico")]
    #[cfg_attr(feature = "pnm", doc = ", ppm, pam")]
    #[cfg_attr(feature = "tga", doc = ", and tga")]
    pub output_path: String,

    #[arg(short, long)]
    /// Print extra information and show the progress of the rendering process
    pub verbose: bool,

    #[arg(short, long)]
    /// The number of parallel jobs to dispatch. If this is not set the program
    /// will let the parallelism library decide.
    pub jobs: Option<NonZeroUsize>,
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
