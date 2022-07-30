use std::error::Error;
use std::io::{stdout, Write};

use crate::{
    config::Args,
    mandelbrot::{render, Frame},
};

use clap::Parser;

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Args::parse();

    let center_real = config.real_center;
    let center_imag = config.imag_center;
    let aspect_ratio = config.aspect_ratio;
    let yresolution = config.pixels;
    let record_params = config.record_params;
    let xresolution = (aspect_ratio * (yresolution as f64)) as u32;
    let zoom = config.zoom;
    let imag_distance = 8.0 / (3.0 * zoom);
    let real_distance = aspect_ratio * imag_distance;
    let ssaa = config.ssaa;

    if ssaa == 0 {
        return Err("SSAA factor must be larger than 0".into());
    }

    //Output some basic information about what the program will be rendering.
    print!("---- Generating a");
    if ssaa != 1 {
        print!(" {} times supersampled", ssaa * ssaa);
    } else {
        print!("n");
    }
    print!(" image with a resolution of {xresolution}x{yresolution}");
    if zoom != 1.0 {
        print!(" zoomed by a factor of {zoom}");
    }
    println!(" ----");

    let draw_region = Frame::new(center_real, center_imag, real_distance, imag_distance);

    //Render the image
    let img = render(xresolution, yresolution, ssaa, draw_region)?;

    print!("\rEncoding and saving image");
    stdout().flush()?;
    let image_name = if record_params {
        format!("re_{center_real}_im_{center_imag}_zoom_{zoom}.png")
    } else {
        "m.png".to_owned()
    };
    img.save(image_name)?;
    println!("\rDone                     ");

    //Everything finished correctly!
    Ok(())
}
