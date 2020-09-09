use std::env;
use std::process;
use std::error::Error;

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
    if let Err(e) = run(config) {
        println!("Application encountered an error: {}", e);

        process::exit(2);
    }
}

//Runs the main logic of the program and returns an error to
//main if something goes wrong
fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let first = config.first;
    let second = config.second;
    println!("The first argument is {}, and the second one is {}", first, second);
    println!("iterate(0,0,255) returns {}", iterate(0.0,0.0,255));
    println!("iterate(10,10,255) returns {}", iterate(10.0,10.0,255));

    //Everything finished correctly!
    Ok(())
}

//Iterates the mandelbrot function on the input number until
//it either escapes or exceeds the maximum number of iterations
fn iterate(c_re: f64, c_im: f64, maxiterations: i32) -> f64 {
    let c_imag_sqr = c_im*c_im;
    let mag_sqr =c_re*c_re + c_imag_sqr;

    //Check whether the point is within the main cardioid or period 2 bulb
    if f64::powf(c_re + 1.0, 2.0) + c_imag_sqr <= 0.0625 || mag_sqr*(8.0*mag_sqr - 3.0) <= 0.09375 - c_re {
        return 0.0
    }
    
    let mut z_re = 0.0;
    let mut z_im = 0.0;
    let mut z_re_sqr = 0.0;
    let mut z_im_sqr = 0.0;
    let mut iterations = 0;
    
    //Iterates the mandelbrot function
    //This loop uses only 3 multiplications, which is the minimum
    while iterations < maxiterations && z_re_sqr + z_im_sqr <= 36.0 {
        z_im = z_re*z_im;
        z_im = z_im + z_im;
        z_im = z_im + c_im;
        z_re = z_re_sqr - z_im_sqr + c_re;
        z_re_sqr = z_re*z_re;
        z_im_sqr = z_im*z_im;
        iterations = iterations + 1
    }

    if iterations == maxiterations {
        return 0.0
    }

    return (maxiterations - iterations) as f64 - 4.0*f64::powf((z_re_sqr + z_im_sqr).sqrt(),-0.4)/(maxiterations as f64)
}

//A struct containing the runtime specified configuration
//of the program
struct Config {
    first: String,
    second: String
}

impl Config {
    //Returns a Result wrapper which contains a Config
    //struct if the arguments could be parsed correctly
    //and an error otherwise
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 + 1 {
            return Err("not enough arguments");
        }
        let first = args[1].clone();
        let second = args[2].clone();
        Ok(Config { first, second})
    }
}

