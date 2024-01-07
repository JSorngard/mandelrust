use core::num::{NonZeroU32, NonZeroU8};

use clap::Parser;

use crate::resolution::Resolution;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Renders a supersampled image of the Mandelbrot set to a png file.
/// It is possible to change which part of the set is rendered, how zoomed in the image is,
/// the number of iterations to use, as well as a few other things.
pub struct Cli {
    // This struct contains the runtime specified configuration of the program.
    #[arg(short, long, value_name = "RE(CENTER)", allow_negative_numbers = true)]
    /// The real part of the center point of the image
    pub real_center: f64,

    #[arg(short, long, value_name = "IM(CENTER)", allow_negative_numbers = true)]
    /// The imaginary part of the center point of the image
    pub imag_center: f64,

    #[arg(short, long, default_value_t = 0.0, allow_negative_numbers = true)]
    /// A real number describing how far in to zoom on the given center point.
    /// This number works on an exponential scale where 0 means no zoom
    /// and every time it is increased by 1 the vertical and horizontal
    /// distances covered by the image are halved
    pub zoom_level: f64,

    #[arg(short = 'p', value_name = "X_RES:Y_RES", long)]
    /// The resolution of the image in the form "X_RES:Y_RES"
    pub resolution: Resolution,

    #[arg(
        short,
        long,
        value_name = "SQRT(SSAA_FACTOR)",
        default_value_t = NonZeroU8::new(3).expect("3 is not 0"),
    )]
    /// How many samples to compute for each pixel along one dimension.
    /// The total number of samples per pixel is the square of this number.
    /// If this is set to 1, supersampling is turned off
    pub ssaa: NonZeroU8,

    #[arg(
        short,
        long,
        default_value_t = NonZeroU32::new(255).expect("255 is not 0"),
    )]
    /// The maximum number of iterations for each pixel sample
    pub max_iterations: NonZeroU32,

    #[arg(long)]
    /// Output the image in grayscale by mapping escape speed to brightness
    pub grayscale: bool,

    #[arg(long)]
    /// Save information about the image location in the complex plane in the file name
    pub record_params: bool,

    #[arg(short, long, default_value = "mandelbrot_renders")]
    /// The folder in which to save the resulting image
    pub output_folder: String,

    #[arg(short, long)]
    /// Print extra information and show the progress of the rendering process
    pub verbose: bool,

    #[arg(short, long, default_value_t = 0)]
    /// The number of parallel jobs to dispatch. If this is 0 the program
    /// will let the parallelism library decide.
    pub jobs: usize,

    #[cfg(feature = "oxipng")]
    #[arg(
        long,
        required = false,
        default_missing_value = "4",
        value_name = "OPTIMIZATION_LEVEL",
        num_args = 0..=1,
        require_equals = true,
        value_parser = clap::value_parser!(u8).range(0..=6),
    )]
    /// Spend extra time after iterations are completed optimizing the file size
    /// of the resulting image. Supports optimizations levels between
    /// 0 and 6. [default level if flag is present: 4]
    pub optimize_file_size: Option<u8>,
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
