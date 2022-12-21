use std::{
    error::Error,
    io::{stdout, Write},
    path::PathBuf,
};

use clap::Parser;

use crate::command_line_interface::Cli;

use mandellib::{render, Frame, RenderParameters};

mod command_line_interface;

const DEFAULT_FILE_NAME: &str = "mandelbrot_set";
const DEFAULT_FILE_EXTENSION: &str = "png";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let x_resolution = (args.aspect_ratio * (args.pixels.get() as f64)) as u32;

    let zoom = 2.0_f64.powf(args.zoom);

    let imag_distance = 8.0 / (3.0 * zoom);
    let real_distance = args.aspect_ratio * imag_distance;

    let draw_region = Frame::new(
        args.real_center,
        args.imag_center,
        real_distance,
        imag_distance,
    );

    let render_parameters = RenderParameters::new(
        x_resolution.try_into()?,
        args.pixels,
        args.max_iterations,
        args.ssaa,
        args.grayscale,
    )?;

    if args.verbose {
        give_user_feedback(&args, &render_parameters)?;
    }

    let img = render(render_parameters, draw_region, args.verbose)?;

    if args.verbose {
        print!("\rEncoding and saving image");
        if stdout().flush().is_err() {
            eprintln!("unable to flush stdout, continuing anyway");
        }
    }

    let image_name = if args.record_params {
        format!(
            "{DEFAULT_FILE_NAME}_at_re_{}_im_{}_zoom_{}_maxiters_{}.{DEFAULT_FILE_EXTENSION}",
            args.real_center, args.imag_center, args.zoom, args.max_iterations,
        )
    } else {
        format!("{DEFAULT_FILE_NAME}.{DEFAULT_FILE_EXTENSION}")
    };

    let mut out_path = PathBuf::new();
    out_path.push(args.output_folder);

    // If the output folder does not exist, we create it
    if !out_path.is_dir() {
        std::fs::create_dir(&out_path)?;
    }
    out_path.push(image_name);

    img.save(&out_path)?;

    if args.verbose {
        println!("\rSaved image as {}", out_path.display());
    }

    // Everything finished correctly!
    Ok(())
}

/// Output some basic information about what the program will be rendering.
fn give_user_feedback(args: &Cli, rparams: &RenderParameters) -> Result<(), Box<dyn Error>> {
    let mut header = Vec::with_capacity(61);
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
        " image with a resolution of {} by {} pixels",
        rparams.x_resolution_u32,
        args.pixels.get(),
    )?;
    if args.zoom > 0.0 {
        write!(
            &mut header,
            " zoomed by a factor of {}",
            2.0_f64.powf(args.zoom)
        )?;
    }
    write!(&mut header, " ----")?;

    println!("{}", std::str::from_utf8(&header)?);

    Ok(())
}
