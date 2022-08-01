use std::error::Error;
use std::io::{stdout, Write};

use crate::{
    config::Args,
    lib::{Frame, RenderParameters},
    mandelbrot::render,
};

use clap::Parser;

mod config;
mod lib;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let center_real = args.real_center;
    let center_imag = args.imag_center;
    let yresolution = args.pixels;
    let zoom = args.zoom;
    let ssaa = args.ssaa;

    let xresolution = (args.aspect_ratio * (yresolution as f64)) as usize;
    let imag_distance = 8.0 / (3.0 * zoom);
    let real_distance = args.aspect_ratio * imag_distance;

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
    print!(" image with a resolution of {xresolution}x{yresolution} pixels");
    if zoom != 1.0 {
        print!(" zoomed by a factor of {zoom}");
    }
    println!(" ----");

    let draw_region = Frame::new(center_real, center_imag, real_distance, imag_distance);

    let render_parameters =
        RenderParameters::new(xresolution, yresolution, 255, ssaa, args.grayscale);

    //Render the image
    let img = render(render_parameters, draw_region)?;

    print!("\rEncoding and saving image");
    stdout().flush()?;
    let image_name = if args.record_params {
        format!("re_{center_real}_im_{center_imag}_zoom_{zoom}.png")
    } else {
        "m.png".to_owned()
    };
    img.save(&image_name)?;
    println!("\rSaved image as {image_name}          ");

    //Everything finished correctly!
    Ok(())
}
