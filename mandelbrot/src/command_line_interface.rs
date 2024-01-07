use core::num::{NonZeroU32, NonZeroU8};

use clap::Parser;

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
    /// How many samples to compute for each pixel along one dimension. The total number of samples per pixel is the square of this number.
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

pub use resolution::Resolution;
mod resolution {
    use core::fmt;
    use core::num::{NonZeroU32, ParseIntError};
    use core::str::FromStr;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Resolution {
        x_res: NonZeroU32,
        y_res: NonZeroU32,
    }

    impl Resolution {
        pub const fn x_resolution(&self) -> NonZeroU32 {
            self.x_res
        }

        pub const fn y_resolution(&self) -> NonZeroU32 {
            self.y_res
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ParseResolutionError {
        InvalidFormat,
        XResInvalidValue(ParseIntError),
        YResInvalidValue(ParseIntError),
        TooLarge,
    }

    impl fmt::Display for ParseResolutionError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidFormat => {
                    write!(f, "the resolution must be given in the format x_res:y_res")
                }
                Self::XResInvalidValue(e) => write!(f, "the x-resolution could not be parsed: {e}"),
                Self::YResInvalidValue(e) => write!(f, "the y-resolution could not be parsed: {e}"),
                Self::TooLarge => {
                    write!(f, "the total number of pixels must be below {}", usize::MAX)
                }
            }
        }
    }

    impl std::error::Error for ParseResolutionError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::XResInvalidValue(e) | Self::YResInvalidValue(e) => Some(e),
                Self::InvalidFormat | Self::TooLarge => None,
            }
        }
    }

    impl FromStr for Resolution {
        type Err = ParseResolutionError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut parts = s.split(':');

            let x_res: NonZeroU32 = match parts.next() {
                Some(s) => s.parse().map_err(|e| Self::Err::XResInvalidValue(e)),
                None => Err(Self::Err::InvalidFormat),
            }?;
            let x_usize: usize = x_res.get().try_into().map_err(|_| Self::Err::TooLarge)?;

            let y_res: NonZeroU32 = match parts.next() {
                Some(s) => s.parse().map_err(|e| Self::Err::YResInvalidValue(e)),
                None => Err(Self::Err::InvalidFormat),
            }?;
            let y_usize: usize = y_res.get().try_into().map_err(|_| Self::Err::TooLarge)?;

            if parts.next().is_some() {
                Err(Self::Err::InvalidFormat)
            } else if x_usize.checked_mul(y_usize).is_none() {
                Err(Self::Err::TooLarge)
            } else {
                Ok(Self { x_res, y_res })
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
