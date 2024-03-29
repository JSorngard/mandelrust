use color_space::SupportedColorType;
use criterion::{criterion_group, criterion_main, Criterion};
use mandellib::{render, Frame, RenderParameters};

fn get_inputs(
    y_res: u32,
    ssaa: Option<u8>,
    zoom: Option<f64>,
    re: Option<f64>,
    im: Option<f64>,
    miters: Option<u32>,
    grayscale: Option<bool>,
) -> (RenderParameters, Frame) {
    let aspect_ratio = 1.5;
    let x_res = (aspect_ratio * y_res as f64) as u32;
    let ssaa = ssaa.unwrap_or(3);
    let grayscale = grayscale.unwrap_or(false);
    let max_iters = miters.unwrap_or(255);

    let params = RenderParameters::try_new(
        x_res.try_into().unwrap(),
        y_res.try_into().unwrap(),
        max_iters.try_into().unwrap(),
        ssaa.try_into().unwrap(),
        if grayscale {
            SupportedColorType::L8
        } else {
            SupportedColorType::Rgb8
        },
    )
    .unwrap();

    let center_real = re.unwrap_or(-0.75);
    let center_imag = im.unwrap_or(0.0);
    let distance_imag = 8.0 / (3.0 * 2.0_f64.powf(zoom.unwrap_or(0.0)));
    let distance_real = aspect_ratio * distance_imag;

    let frame = Frame::new(center_real, center_imag, distance_real, distance_imag);

    (params, frame)
}

fn fast(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast renders");

    let (params, frame) = get_inputs(480, None, None, None, None, None, None);
    group.bench_function(
        &format!(
            "{}x{} render of full set",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );

    let (params, frame) = get_inputs(720, None, None, None, None, None, None);
    group.bench_function(
        &format!(
            "{}x{} render of full set",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );

    let (params, frame) = get_inputs(1080, None, None, None, None, None, None);
    group.bench_function(
        &format!(
            "{}x{} render of full set",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );

    let (params, frame) = get_inputs(1080, None, None, None, None, None, Some(true));
    group.bench_function(
        &format!(
            "{}x{} grayscale render of full set",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );

    let (params, frame) = get_inputs(1080, Some(1), None, None, None, None, None);
    group.bench_function(
        &format!(
            "{}x{} render  of full set without SSAA",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );
}

fn slow(c: &mut Criterion) {
    let mut group = c.benchmark_group("slow renders");
    group.sample_size(10);

    let (params, frame) = get_inputs(2160, None, None, None, None, None, None);
    group.bench_function(
        &format!(
            "{}x{} render of full set",
            params.x_resolution, params.y_resolution
        ),
        |b| b.iter(|| render(params, frame, false)),
    );

    let zoom = 12.0;
    let (params, frame) = get_inputs(
        1080,
        None,
        Some(zoom),
        Some(-0.2345),
        Some(-0.7178),
        Some(1000),
        None,
    );

    group.bench_function(
        &format!(
            "{}x{}, {} iterations, zoomed by 2^{}: 'Mandelsun'",
            params.x_resolution, params.y_resolution, params.max_iterations, zoom
        ),
        |b| b.iter(|| render(params, frame, false)),
    );
}

criterion_group!(benches, fast, slow);
criterion_main!(benches);
