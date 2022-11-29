use core::num::{NonZeroU32, NonZeroU8, NonZeroUsize};

use criterion::{criterion_group, criterion_main, Criterion};
use mandelbrot::{render, Frame, RenderParameters};

fn get_inputs(y_res: usize, ssaa: u8, zoom: f64) -> (RenderParameters, Frame) {
    let aspect_ratio = 1.5;
    let x_res = NonZeroUsize::new((aspect_ratio * y_res as f64) as usize).unwrap();
    let y_res = NonZeroUsize::new(y_res).unwrap();
    let ssaa = NonZeroU8::new(ssaa).unwrap();
    let grayscale = false;
    let max_iters = NonZeroU32::new(255).unwrap();

    let params = RenderParameters::new(x_res, y_res, max_iters, ssaa, grayscale);

    let center_real = -0.75;
    let center_imag = 0.0;
    let distance_imag = 8.0 / (3.0 * zoom);
    let distance_real = aspect_ratio * distance_imag;

    let frame = Frame::new(center_real, center_imag, distance_real, distance_imag);

    (params, frame)
}

fn sd(c: &mut Criterion) {
    let (params, frame) = get_inputs(480, 3, 1.0);

    c.bench_function(
        &format!("{}x{} render", params.x_resolution, params.y_resolution),
        |b| b.iter(|| render(params, frame, false).unwrap()),
    );
}

fn hd(c: &mut Criterion) {
    let (params, frame) = get_inputs(720, 3, 1.0);

    c.bench_function(
        &format!("{}x{} render", params.x_resolution, params.y_resolution),
        |b| b.iter(|| render(params, frame, false).unwrap()),
    );
}

fn full_hd(c: &mut Criterion) {
    let (params, frame) = get_inputs(1080, 3, 1.0);

    c.bench_function(
        &format!("{}x{} render", params.x_resolution, params.y_resolution),
        |b| b.iter(|| render(params, frame, false).unwrap()),
    );
}

fn fourk(c: &mut Criterion) {
    let (params, frame) = get_inputs(2160, 3, 1.0);

    c.bench_function(
        &format!("{}x{} render", params.x_resolution, params.y_resolution),
        |b| b.iter(|| render(params, frame, false).unwrap()),
    );
}

fn no_ssaa_full_hd(c: &mut Criterion) {
    let (params, frame) = get_inputs(1080, 1, 1.0);
    c.bench_function(
        &format!(
            "{}x{} without SSAA",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false).unwrap()),
    );
}

criterion_group!(benches, sd, hd, full_hd, fourk, no_ssaa_full_hd);
criterion_main!(benches);
