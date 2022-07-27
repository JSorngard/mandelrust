use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    mandelrust::run(mandelrust::Config::new()?)?;
    Ok(())
}