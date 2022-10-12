# mandelrust
Renders a supersampled image of the Mandelbrot set to a png file. It is possible to change which part of the set is rendered, how zoomed in the image is, as well as the number of iterations to use.

This was one of my first projects to learn rust. It contains some file manipulation and concurrency.

# How to use this program
 0. Install Rust
 1. Clone this git repo and build with `cargo build --release`
 2. Run the program with `./target/release/mandelrust.exe`
 3. You can specify where the image is focused, how zoomed it is and how many iterations to do (among other things) with command line arguments. For an exhaustive list run the program with the `--help` argument