# mandelrust
Renders a supersampled image of the Mandelbrot set to a png file. It is possible to change which part of the set is rendered, how zoomed in the image is, the number of iterations to use, as well as a few other things.

This was one of my first projects to learn rust.

# Faster mandelbrot rendering
I have tried to make the program faster over time. Some of the techniques used are

 1. If the image contains the real axis, split the image there and only render the larger half. Then mirror the smaller half from it.  
 2. There are [closed form expressions](https://en.wikipedia.org/wiki/Mandelbrot_set#Main_cardioid_and_period_bulbs) for checking if a point is inside the main cardioid or the period 2 bulb, these are used to skip a large amount of iteration.  
 3. [Rayon](https://docs.rs/rayon/latest/rayon/) is used to parallelize the rendering.  
 4. Supersampling is done only close to the border of the set.  
 5. The iteration loop has been restructured to use the minimum number of multiplications.  
 6. Link time optimization has been enabled, and the number of codegen units set to 1.  
 7. Cargo is set to enable optimization with every instruction set available on the compiling CPU.  

The program can render a nine times supersampled 8k image of the set in just over four seconds on my laptop with an i7-7700U CPU.

# How to use this program
 0. Install Rust
 1. Clone this git repo and build with `cargo build --release`
 2. Run the program with `./target/release/mandelrust.exe`
 3. You can specify where the image is focused, how zoomed it is and how many iterations to do (among other things) with command line arguments. For an exhaustive list run the program with the `--help` argument

# Example images
![Full set](/examples/mandelbrot_set.png)
![Zoomed detail](/examples/mandelbrot_set_at_re_-0.23_im_-0.72_zoom_85_maxiters_255.png)
![Deep zoomed detail](/examples/mandelbrot_set_at_re_-0.2345_im_-0.7178_zoom_6000_maxiters_1000.png)
![Grayscale example](/examples/mandelbrot_set_at_re_-0.728_im_-0.212_zoom_85_maxiters_1000.png)