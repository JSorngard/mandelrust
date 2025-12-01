#![forbid(unsafe_code)]

// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use core::str;

use clap::Parser;
use color_space::SupportedColorType;
use rayon::ThreadPoolBuilder;

use crate::command_line_interface::Cli;

use mandellib::{render, Frame, RenderParameters};

mod command_line_interface;
mod resolution;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let x_resolution = args.resolution.x_resolution();
    let y_resolution = args.resolution.y_resolution();

    let zoom = 2.0_f64.powf(args.zoom_level);

    let imag_distance = 8.0 / (3.0 * zoom);
    let real_distance =
        f64::from(x_resolution.get()) / f64::from(y_resolution.get()) * imag_distance;

    let draw_region = Frame::new(
        args.real_center,
        args.imag_center,
        real_distance,
        imag_distance,
    );

    let render_parameters = RenderParameters::try_new(
        x_resolution,
        y_resolution,
        args.max_iterations,
        args.ssaa,
        if args.grayscale {
            SupportedColorType::L8
        } else {
            SupportedColorType::Rgb8
        },
    )?;

    if args.verbose {
        _ = give_user_feedback(&args, &render_parameters);
    }

    if let Some(jobs) = args.jobs {
        ThreadPoolBuilder::new()
            .num_threads(jobs.into())
            .build_global()?;
    }

    let img = render(render_parameters, draw_region, args.verbose);

    if args.verbose {
        _ = write!(io::stdout(), "\rEncoding and saving image");
    }

    let out_path = PathBuf::from(args.output_path);

    img.save(&out_path)?;

    if args.verbose {
        _ = writeln!(
            io::stdout(),
            "\rSaved image as {}                       ",
            out_path.display()
        );
    }

    Ok(())
}

/// Output some basic information about what the program will be rendering.
fn give_user_feedback(args: &Cli, rparams: &RenderParameters) -> Result<(), Box<dyn Error>> {
    let mut header = Vec::with_capacity(80);
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
        u32::from(rparams.x_resolution),
        rparams.y_resolution,
    )?;
    if args.zoom_level > 0.0 {
        write!(
            &mut header,
            " zoomed by a factor of {}",
            2.0_f64.powf(args.zoom_level)
        )?;
    }
    write!(&mut header, " ----")?;

    writeln!(io::stdout(), "{}", str::from_utf8(&header)?)?;

    Ok(())
}
