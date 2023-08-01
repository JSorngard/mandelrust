use core::num::{NonZeroU32, NonZeroU8};
use std::num::NonZeroUsize;

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
        allow_negative_numbers = true,
    )]
    /// The real part of the center point of the image
    pub real_center: f64,

    #[arg(
        short,
        long,
        value_name = "IM(CENTER)",
        default_value_t = 0.0,
        allow_negative_numbers = true
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
        short,
        long,
        value_parser(parse_resolution),
        default_value_t = NonZeroU32::new(2160).expect("2160 is not 0"),
    )]
    /// The number of pixels along the y-axis of the image
    pub pixels: NonZeroU32,

    #[arg(short, long, default_value_t = AspectRatio::Number(1.5))]
    /// The aspect ratio of the image. The horizontal pixel resolution is calculated by multiplying the
    /// vertical pixel resolution by this number. The aspect ratio can be entered as a real number and also in the format x:y,
    /// where x and y are integers, e.g. 3:2
    pub aspect_ratio: AspectRatio,

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

pub use aspect_ratio::AspectRatio;
mod aspect_ratio {
    use core::{fmt, num::ParseIntError, ops::Mul, str::FromStr};
    use std::error::Error;

    use super::NonZeroU32;

    #[derive(Debug, Clone, Copy)]
    pub enum AspectRatio {
        Number(f64),
        Ratio(NonZeroU32, NonZeroU32),
    }

    impl core::convert::From<AspectRatio> for f64 {
        fn from(value: AspectRatio) -> Self {
            match value {
                AspectRatio::Number(r) => r,
                AspectRatio::Ratio(x, y) => f64::from(x.get()) / f64::from(y.get()),
            }
        }
    }

    impl Mul<f64> for AspectRatio {
        type Output = f64;
        fn mul(self, rhs: f64) -> Self::Output {
            f64::from(self) * rhs
        }
    }

    impl fmt::Display for AspectRatio {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Number(r) => write!(f, "{r}"),
                Self::Ratio(x, y) => write!(f, "{x}:{y}"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum ParseAspectRatioError {
        NonPositive,
        ParseNumerator(ParseIntError),
        ParseDenominator(ParseIntError),
        ParseBoth(ParseTwoIntError),
        InvalidFormat,
    }

    impl fmt::Display for ParseAspectRatioError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NonPositive => write!(f, "aspect ratio must be larger than zero"),
                Self::ParseNumerator(e) => write!(f, "Horizontal integer has issue: {e}"),
                Self::ParseDenominator(e) => write!(f, "Vertical integer has issue: {e}"),
                Self::ParseBoth(e) => write!(
                    f,
                    "horizontal integer has issue: '{}' vertical integer has issue '{}'",
                    e.e1, e.e2
                ),
                Self::InvalidFormat => {
                    write!(f, "input could not be interpreted as an aspect ratio")
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ParseTwoIntError {
        e1: ParseIntError,
        e2: ParseIntError,
    }

    impl From<(ParseIntError, ParseIntError)> for ParseTwoIntError {
        fn from(value: (ParseIntError, ParseIntError)) -> Self {
            Self {
                e1: value.0,
                e2: value.1,
            }
        }
    }

    impl fmt::Display for ParseTwoIntError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}, {}", self.e1, self.e2)
        }
    }

    impl Error for ParseTwoIntError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    impl Error for ParseAspectRatioError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::NonPositive | Self::InvalidFormat => None,
                Self::ParseNumerator(e) | Self::ParseDenominator(e) => Some(e),
                Self::ParseBoth(e) => Some(e),
            }
        }
    }

    impl FromStr for AspectRatio {
        type Err = ParseAspectRatioError;
        /// Tries to interpret the input string as if it is an aspect ratio.
        /// 3:2 and 1.5 both work.
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Ok(x_by_y) = s.parse::<f64>() {
                if x_by_y > 0.0 {
                    Ok(Self::Number(x_by_y))
                } else {
                    Err(Self::Err::NonPositive)
                }
            } else {
                let substrings: Vec<&str> = s.split(':').collect();
                if substrings.len() == 2 {
                    match (
                        substrings[0].parse::<NonZeroU32>(),
                        substrings[1].parse::<NonZeroU32>(),
                    ) {
                        (Ok(x), Ok(y)) => Ok(Self::Ratio(x, y)),
                        (Ok(_), Err(e)) => Err(Self::Err::ParseDenominator(e)),
                        (Err(e), Ok(_)) => Err(Self::Err::ParseNumerator(e)),
                        (Err(e1), Err(e2)) => Err(Self::Err::ParseBoth((e1, e2).into())),
                    }
                } else {
                    Err(Self::Err::InvalidFormat)
                }
            }
        }
    }
}

fn parse_resolution(s: &str) -> Result<NonZeroU32, String> {
    let candidate: NonZeroU32 = match s.parse() {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };

    if NonZeroUsize::try_from(candidate).is_err() {
        return Err("given resolution would not fit in both a u32 and usize".to_owned());
    };

    Ok(candidate)
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
