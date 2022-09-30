use std::{
    error::Error,
    io::{stdout, Write},
};

use clap::Parser;

use crate::{
    config::Cli,
    mandelbrot::render,
    structs::{Frame, RenderParameters},
};

mod config;
mod mandelbrot;
mod structs;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let center_real = args.real_center;
    let center_imag = args.imag_center;
    let yresolution = args.pixels.get();
    let zoom = args.zoom;
    let ssaa = args.ssaa.get();
    let max_iterations = args.max_iterations.get();
    let aspect_ratio = args.aspect_ratio;

    let xresolution = (aspect_ratio * (yresolution as f64)) as usize;
    let imag_distance = 8.0 / (3.0 * zoom);
    let real_distance = args.aspect_ratio * imag_distance;

    let draw_region = Frame::new(center_real, center_imag, real_distance, imag_distance);

    let render_parameters = RenderParameters::new(
        xresolution,
        yresolution,
        max_iterations,
        ssaa,
        args.grayscale,
    );

    //Output some basic information about what the program will be rendering.
    let mut header = Vec::new();
    write!(&mut header, "---- Generating a")?;
    if ssaa == 1 {
        write!(&mut header, "n")?;
    } else {
        write!(&mut header, " {} times supersampled", ssaa * ssaa)?;
    }
    write!(
        &mut header,
        " image with a resolution of {xresolution} by {yresolution} pixels"
    )?;
    if (zoom - 1.0).abs() > f64::EPSILON {
        write!(&mut header, " zoomed by a factor of {zoom}")?;
    }
    write!(&mut header, " ----")?;

    println!("{}", std::str::from_utf8(&header)?);

    //Render the image
    let img = render(render_parameters, draw_region)?;

    print!("\rEncoding and saving image");
    stdout().flush()?;
    let image_name = if args.record_params {
        format!("mandelbrot_set_at_re_{center_real}_im_{center_imag}_zoom_{zoom}_maxiters_{max_iterations}.png")
    } else {
        "mandelbrot_set.png".to_owned()
    };
    img.save(&image_name)?;
    println!("\rSaved image as {image_name}          ");

    //Everything finished correctly!
    Ok(())
}
