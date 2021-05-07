extern crate image;

fn main() {
    //Pass the command line parameters from the environment
    //into the constructor of the config function. This then
    //returns a config object, or an error if the input can
    //not be parsed
    let config = mandelrust::Config::new().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    //Call the run function, and if it returns an error,
    //display it
    if let Err(e) = mandelrust::run(config) {
        eprintln!("Application encountered an error: {}", e);
        std::process::exit(1);
    }
}
