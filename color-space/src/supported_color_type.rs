use image::ColorType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedColorType {
    Rgba8,
    Rgb8,
    L8,
}

impl From<SupportedColorType> for ColorType {
    fn from(sct: SupportedColorType) -> Self {
        match sct {
            SupportedColorType::L8 => ColorType::L8,
            SupportedColorType::Rgb8 => ColorType::Rgb8,
            SupportedColorType::Rgba8 => ColorType::Rgba8,
        }
    }
}

impl SupportedColorType {
    #[must_use]
    pub fn bytes_per_pixel(&self) -> u8 {
        ColorType::from(*self).bytes_per_pixel()
    }

    #[must_use]
    pub fn has_color(&self) -> bool {
        ColorType::from(*self).has_color()
    }

    #[must_use]
    pub fn has_alpha(&self) -> bool {
        ColorType::from(*self).has_alpha()
    }

    #[must_use]
    pub fn channel_count(&self) -> u8 {
        ColorType::from(*self).channel_count()
    }

    #[must_use]
    pub fn bits_per_pixel(&self) -> u16 {
        ColorType::from(*self).bits_per_pixel()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedColorTypeError {
    La8,
    L16,
    La16,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
    Unknown,
}

impl std::fmt::Display for UnsupportedColorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is not supported",
            match self {
                Self::La8 => "LA8",
                Self::L16 => "L16",
                Self::La16 => "LA16",
                Self::Rgb16 => "RGB16",
                Self::Rgba16 => "RGBA16",
                Self::Rgb32F => "RGB32F",
                Self::Rgba32F => "RGBA32F",
                Self::Unknown => "<unknown color type>",
            }
        )
    }
}

impl std::error::Error for UnsupportedColorTypeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl TryFrom<ColorType> for SupportedColorType {
    type Error = UnsupportedColorTypeError;
    fn try_from(value: ColorType) -> Result<Self, Self::Error> {
        match value {
            ColorType::L8 => Ok(Self::L8),
            ColorType::Rgb8 => Ok(Self::Rgb8),
            ColorType::Rgba8 => Ok(Self::Rgba8),
            ColorType::La8 => Err(UnsupportedColorTypeError::La8),
            ColorType::L16 => Err(UnsupportedColorTypeError::L16),
            ColorType::La16 => Err(UnsupportedColorTypeError::La16),
            ColorType::Rgb16 => Err(UnsupportedColorTypeError::Rgb16),
            ColorType::Rgba16 => Err(UnsupportedColorTypeError::Rgba16),
            ColorType::Rgb32F => Err(UnsupportedColorTypeError::Rgb32F),
            ColorType::Rgba32F => Err(UnsupportedColorTypeError::Rgba32F),
            _ => Err(UnsupportedColorTypeError::Unknown),
        }
    }
}
