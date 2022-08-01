#[derive(Clone, Copy, Debug, Default)]
pub struct Frame {
    pub center_real: f64,
    pub center_imag: f64,
    pub real_distance: f64,
    pub imag_distance: f64,
}

impl Frame {
    pub fn new(center_real: f64, center_imag: f64, real_distance: f64, imag_distance: f64) -> Self {
        Frame {
            center_real,
            center_imag,
            real_distance,
            imag_distance,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderParameters {
    pub x_resolution: usize,
    pub y_resolution: usize,
    pub iterations: u32,
    pub ssaa: u8,
    pub grayscale: bool,
}

impl RenderParameters {
    pub fn new(x_resolution: usize, y_resolution: usize, iterations: u32, ssaa: u8, grayscale: bool) -> Self {
        RenderParameters {
            x_resolution,
            y_resolution,
            iterations,
            ssaa,
            grayscale,
        }
    }
}
