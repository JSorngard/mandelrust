use crate::config::Config;
use crate::mandelbrot::{render, Frame};
use std::error::Error;
use std::io::{stdout, Write};

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    let center_real = config.center_real;
    let center_imag = config.center_imag;
    let aspect_ratio = config.aspect_ratio;
    let yresolution = config.resolution;
    let save_result = config.save_result;
    let record_params = config.record_params;
    let xresolution = (aspect_ratio * (yresolution as f64)) as u32;
    let zoom = config.zoom;
    let imag_distance = config.imag_distance / zoom;
    let real_distance = aspect_ratio * imag_distance;
    let ssaa = config.ssaa;

    //Output some basic information about what the program will be rendering.
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
        print!(" zoomed by a factor of {}", zoom);
    }
    println!(" ----");

    let draw_region = Frame::new(center_real, center_imag, real_distance, imag_distance);

    //Render the image
    let img = render(xresolution, yresolution, ssaa, draw_region)?;

    if save_result {
        print!("\rEncoding and saving image");
        stdout().flush()?;
        let image_name = if record_params {
            format!("re_{center_real}_im_{center_imag}_zoom_{zoom}.png")
        } else {
            "m.png".to_owned()
        };
        img.save(image_name)?;
    }
    println!("\rDone                     ");

    //Everything finished correctly!
    Ok(())
}
