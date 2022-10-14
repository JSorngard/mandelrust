use std::{
    error::Error,
    io::{stdout, Write},
    path::PathBuf,
};

use clap::Parser;

use crate::{
    config::Cli,
    mandelbrot::{render, Frame, RenderParameters},
};

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let xresolution = (args.aspect_ratio * (args.pixels.get() as f64)) as usize;

    let imag_distance = 8.0 / (3.0 * args.zoom);
    let real_distance = args.aspect_ratio * imag_distance;

    let draw_region = Frame::new(
        args.real_center,
        args.imag_center,
        real_distance,
        imag_distance,
    );

    let render_parameters = RenderParameters::new(
        xresolution.try_into()?,
        args.pixels,
        args.max_iterations,
        args.ssaa,
        args.grayscale,
    );

    // Output some basic information about what the program will be rendering.
    let mut header = Vec::new();
    write!(&mut header, "---- Generating a")?;
    if args.ssaa.get() == 1 {
        write!(&mut header, "n")?;
    } else {
        write!(
            &mut header,
            " {} times supersampled",
            u16::from(args.ssaa.get()) * u16::from(args.ssaa.get())
        )?;
    }
    write!(
        &mut header,
        " image with a resolution of {xresolution} by {} pixels",
        args.pixels.get(),
    )?;
    if (args.zoom - 1.0).abs() > f64::EPSILON {
        write!(&mut header, " zoomed by a factor of {}", args.zoom)?;
    }
    write!(&mut header, " ----")?;

    println!("{}", std::str::from_utf8(&header)?);

    // Render the image
    let img = render(render_parameters, draw_region)?;

    print!("\rEncoding and saving image");
    stdout().flush()?;
    let image_name = if args.record_params {
        format!(
            "mandelbrot_set_at_re_{}_im_{}_zoom_{}_maxiters_{}.png",
            args.real_center, args.imag_center, args.zoom, args.max_iterations
        )
    } else {
        "mandelbrot_set.png".to_owned()
    };

    let mut out_path = PathBuf::new();
    out_path.push(args.output_folder);

    // If the output folder does not exist, we create it
    if !out_path.is_dir() {
        std::fs::create_dir(&out_path)?;
    }
    out_path.push(image_name);

    img.save(&out_path)?;
    println!("\rSaved image as {}", out_path.display());

    // Everything finished correctly!
    Ok(())
}
