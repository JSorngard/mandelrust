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
