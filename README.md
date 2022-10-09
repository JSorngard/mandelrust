# mandelrust
Renders a supersampled image of the Mandelbrot set to a png file. It is possible to change which part of the set is rendered, how zoomed in the image is, as well as the number of iterations to use.

This was one of my first projects to learn rust. It contains some file manipulation and concurrency.

I have tried to make it faster over time, and now on my laptop with an i7-7500U CPU it renders and saves a nine times supersampled 8k png image of the set in around 4 seconds.

The main techniques used for speeding up the computation are:
 1. Save on processing time by only iterating ~half the pixels in the image, and then use the fact that the set is symmetric under conjugation to produce the other half.
 2. Hard code a test to check if the pixel is within the main cardioid or period-2-bulb and return immediately to skip those pixels.
 3. Re-express the mandelbrot function to use as few multiplications as possible in the iteration loop.
 4. Use rayon for multithreaded computation, as this is an embarrassingly parallel task.

It's not as fast as it can be, mostly because it also computes color curves from the iteration data that uses slow math operations and supersamples every pixel (nine times by default). These functions can be turned off with command line arguments.