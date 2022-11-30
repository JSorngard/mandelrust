# mandelrust
Renders a supersampled image of the Mandelbrot set to a png file. It is possible to change which part of the set is rendered, how zoomed in the image is, the number of iterations to use, as well as a few other things.

This was one of my first projects to learn rust.

# How to use this program
 1. Install [Rust](https://www.rust-lang.org/tools/install) and [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
 2. Open a terminal in the folder you want to install the program in
 3. Clone this git repo with `git clone https://www.github.com/JSorngard/mandelrust.git`
 4. Go into the repository with `cd mandelrust`
 5. Compile the program with `cargo build --release`
 6. Run the program with `./target/release/mandelrust.exe`. The resulting image can be found in the `renders` folder.
 7. You can specify where the image is focused, how zoomed it is and how many iterations to do (among other things) with command line arguments. For an exhaustive list run the program with the `--help` argument

# Prettier mandelbrot rendering
This is of course subjective, but here is a list of what I've done to make the resulting images look better in my eyes:  

 1. Use a color palette that is smooth i.e. small differences in escape speed should map to small differences in color. In this program this is achieved by the color palette being a continuous function that maps escape speeds to colors.  
 2. Do not abort the iteration [when |z| > 2](/examples/smoothing/no_smoothing_abs_geq_2.avif), but at a larger absolute value (in this program I have [chosen 6](/examples/smoothing/no_smoothing_abs_geq_6.avif)). Together with using a function that smoothly maps iteration count and absolute value to a number between 0 and 1 this [completely removes color banding](/examples/smoothing/smoothing_abs_geq_6.avif).  
 4. [Supersample the image](/examples/smoothing/smoothing_abs_geq_6_ssaa.avif) to remove graininess.

# Faster mandelbrot rendering
I have tried to make the program faster over time. Some of the techniques used are:

 1. If the image contains the real axis, split the image there and only render the larger half, then mirror the smaller half from it.
 2. There are [closed form expressions](https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Cardioid_/_bulb_checking) for checking if a point is inside the main cardioid or the period 2 bulb, these are used to skip a large amount of iteration.  
 3. [Rayon](https://docs.rs/rayon/latest/rayon/) is used to parallelize the rendering.  
 4. Supersampling is done only [close to the border of the set](/examples/smoothing/mandelbrot_set_ssaa_region.avif).  
 5. The iteration loop has been restructured to use the minimum number of multiplications.  
 6. [Link time optimization](https://doc.rust-lang.org/rustc/codegen-options/index.html#lto) has been enabled, the number of [codegen units](https://doc.rust-lang.org/rustc/codegen-options/index.html#codegen-units) set to 1, and the [opt-level](https://doc.rust-lang.org/rustc/codegen-options/index.html#opt-level) to 3.  
 7. Cargo is set to enable optimization with every instruction set available [on the compiling CPU](https://rust-lang.github.io/packed_simd/perf-guide/target-feature/rustflags.html#target-cpu).

The program can render a nine times supersampled 4k image of the set in just over 800 ms on my laptop with a quad core i7-7500U CPU, while a non-supersampled 1080p image finishes in around 150 ms.

You can easily test the performance of various renders on your own machine with `cargo bench`. The results will be printed to the terminal, but more detailed data can be found in `target/criterion/report/index.html` in the form of a web page. Note that this currently only benchmarks renders of the fully zoomed out default image, no zoomed renders are benchmarked.

# Example images
Default settings:
![Full set, default settings](/examples/mandelbrot_set.avif)
We can zoom in on details in the above image:
![Zoomed detail without changing max iterations](/examples/mandelbrot_set_at_re_-0.23_im_-0.72_zoom_6.4_maxiters_255.avif)
If we want to zoom even deeper we can change the maximum number of iterations in order to keep the image crisp:
![Deep zoomed detail with changed max iterations](/examples/mandelbrot_set_at_re_-0.2345_im_-0.7178_zoom_12.5_maxiters_1000.avif)
Images in grayscale without any color mapping can also be made:
![Grayscale example](/examples/mandelbrot_set_at_re_-0.728_im_-0.212_zoom_6.4_maxiters_1000.avif)
