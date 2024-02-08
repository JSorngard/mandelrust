use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
/// This program displays a graphical user interface that lets you view the mandelbrot fractal.
pub struct Cli {
    /// The number of parallel threads to launch when rendering.
    /// This is a global setting and can not be changed after program start.
    /// If this is not given the program lets the parallelism library decide.
    #[arg(short, long)]
    pub jobs: Option<core::num::NonZeroUsize>,
}
