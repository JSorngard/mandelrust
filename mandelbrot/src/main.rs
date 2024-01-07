use std::{
    error::Error,
    fs,
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

const DEFAULT_FILE_NAME: &str = "mandelbrot_set";
const DEFAULT_FILE_EXTENSION: &str = "png";

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

    if args.jobs > 0 {
        ThreadPoolBuilder::new()
            .num_threads(args.jobs)
            .build_global()?;
    }

    let img = render(render_parameters, draw_region, args.verbose);

    if args.verbose {
        _ = write!(io::stdout(), "\rEncoding and saving image");
    }

    let image_name = if args.record_params {
        format!(
            "{DEFAULT_FILE_NAME}_at_re_{}_im_{}_zoom_{}_maxiters_{}.{DEFAULT_FILE_EXTENSION}",
            args.real_center, args.imag_center, args.zoom_level, args.max_iterations,
        )
    } else {
        format!("{DEFAULT_FILE_NAME}.{DEFAULT_FILE_EXTENSION}")
    };

    let mut out_path = PathBuf::new();
    out_path.push(args.output_folder);

    // If the output folder does not exist, we create it
    if !out_path.is_dir() {
        fs::create_dir(&out_path)?;
    }
    out_path.push(image_name);

    img.save(&out_path)?;

    #[cfg(feature = "oxipng")]
    if let Some(level) = args.optimize_file_size {
        use oxipng::{optimize, InFile, Options, OutFile};
        if args.verbose {
            _ = write!(io::stdout(), "\rOptimizing output file   ");
            _ = io::stdout().flush();
        }
        optimize(
            &InFile::Path(out_path.clone()),
            &OutFile::Path {
                path: None,
                preserve_attrs: true,
            },
            &Options::from_preset(level),
        )?;
    }

    if args.verbose {
        _ = writeln!(io::stdout(), "\rSaved image as {}", out_path.display());
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
