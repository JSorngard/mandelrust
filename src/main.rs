use std::error::Error;
use crate::config::Config;
use crate::mandelbrot::run;

mod config;
mod mandelbrot;

fn main() -> Result<(), Box<dyn Error>> {
    run(Config::new()?)?;
    Ok(())
}