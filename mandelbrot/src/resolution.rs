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
