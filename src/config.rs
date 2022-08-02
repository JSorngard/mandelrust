use clap::Parser;

///Renders an image of the Mandelbrot set
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    //This struct contains the runtime specified configuration of the program.
    #[clap(short, long, value_parser, value_name = "RE(CENTER)", default_value_t = -0.75, help= "The real part of the center point of the image", allow_hyphen_values = true)]
    pub real_center: f64,

    #[clap(
        short,
        long,
        value_parser,
        value_name = "IM(CENTER)",
        default_value_t = 0.0,
        help = "The imaginary part of the center point of the image",
        allow_hyphen_values = true
    )]
    pub imag_center: f64,

    #[clap(
        short,
        long,
        value_parser,
        default_value_t = 1.5,
        help = "The aspect ratio of the image"
    )]
    pub aspect_ratio: f64,

    #[clap(
        short,
        long,
        value_parser,
        default_value_t = 2160,
        help = "The number of pixels along the y-axis of the image"
    )]
    pub pixels: usize,

    #[clap(
        short,
        long,
        value_parser,
        value_name = "SQRT(SSAA FACTOR)",
        default_value_t = 3,
        help = "How many samples to compute for each pixel (along one direction, the actual number of samples is the square of this number)"
    )]
    pub ssaa: u8,

    #[clap(long, help = "Output the image in grayscale instead of color")]
    pub grayscale: bool,

    #[clap(
        long,
        help = "Save information about the image location in the complex plane in the file name"
    )]
    pub record_params: bool,

    #[clap(
        short,
        long,
        value_parser,
        value_name = "ZOOM LEVEL",
        default_value_t = 1.0,
        help = "How far in to zoom on the given center point"
    )]
    pub zoom: f64,
}
