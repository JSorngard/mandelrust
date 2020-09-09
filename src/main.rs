use std::process;
use std::env;

use rustybrot::Config;

fn main() {
    
    //Define a vector to hold the arguments from the command line
    let args: Vec<String> = env::args().collect();

    //Use the vector to create a new Config struct
    //if it can not be parsed, display the resulting
    //error
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(2);
    });

    //Call the run function, and if it returns an error,
    //display it
    if let Err(e) = rustybrot::run(config) {
        println!("Application encountered an error: {}", e);

        process::exit(2);
    }
}