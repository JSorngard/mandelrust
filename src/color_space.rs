use lazy_static::lazy_static;

lazy_static! {
    pub static ref SRGB_TO_LINEAR: Vec<f64> = (0..=u8::MAX).map(|c_srgb| {
        let c_linear = f64::from(c_srgb)/255.0;
        if c_linear <= 0.04045 {
            c_linear / 12.92
        } else {
            ((c_linear + 0.055) / 1.055).powf(2.4)
        }
    }).collect();

    pub static ref LINEAR_TO_SRGB: Vec<f64> = (0..=u8::MAX).map(|c_linear| {
        let c_srgb = f64::from(c_linear)/255.0;
        if c_srgb <= 0.0031308 {
            12.92 * c_srgb
        } else {
            1.055 * c_srgb.powf(1.0 / 2.4) - 0.055
        }
    }).collect();
}

/// Converts a point in the sRGB color space to a linear RGB triplet.
pub fn srgb_to_linear_rgb(srgb: [f64; 3]) -> [f64; 3] {
    srgb.map(|c| c * c) // <-- approximation of the below

    // srgb.map(|c| {
    //     if c <= 0.04045 {
    //         c / 12.92
    //     } else {
    //         ((c + 0.055) / 1.055).powf(2.4)
    //     }
    // })
}

/// Converts an RGB triplet into a point in the sRGB color space.
pub fn linear_rgb_to_srgb(linear_rgb: [f64; 3]) -> [f64; 3] {
    linear_rgb.map(|c| c.sqrt()) // <-- approximation of the below

    // linear_rgb.map(|c| {
    //     if c <= 0.0031308 {
    //         12.92 * c
    //     } else {
    //         1.055 * c.powf(1.0 / 2.4) - 0.055
    //     }
    // })
}
