use crate::config::Config;
use crate::mandelbrot::render;
use std::error::Error;
use std::io::Write;

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    let center_real = config.center_real;
    let center_imag = config.center_imag;
    let aspect_ratio = config.aspect_ratio;
    let yresolution = config.resolution;
    let save_result = config.save_result;
    let xresolution = (aspect_ratio * (yresolution as f64)) as u32;
    let zoom = config.zoom;
    let imag_distance = config.imag_distance / zoom;
    let real_distance = aspect_ratio * imag_distance;
    let ssaa = config.ssaa;
    let verbose = config.verbose;

    //Output some basic information about what the program will be rendering.
    if verbose {
        print!("---- Generating a");
        if ssaa != 1 {
            print!(" {} times supersampled", u32::pow(ssaa, 2));
        } else {
            print!("n");
        }
        print!(
            " image with a resolution of {}x{}",
            xresolution, yresolution
        );
        if zoom != 1.0 {
            print!("zoomed by a factor of {}", zoom);
        }
        println!(" ----");
    }

    let img = render(
        xresolution,
        yresolution,
        ssaa,
        center_real,
        center_imag,
        real_distance,
        imag_distance,
        verbose,
    )?;

    if save_result {
        if verbose {
            print!("\rEncoding and saving image    ");
            flush()?;
        }
        img.save("m.png")?;
    }
    if verbose {
        println!("\rDone                     ");
    }

    //Everything finished correctly!
    Ok(())
}

//Flushes the stdout buffer.
fn flush() -> Result<(), std::io::Error> {
    std::io::stdout().flush()
}