use crate::config::Config;
use crate::mandelbrot::run;
use std::error::Error;

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    run(Config::new()?)?;
    Ok(())
}
