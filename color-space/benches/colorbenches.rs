use color_space::{palette, LinearRGB};
use criterion::{criterion_group, criterion_main, Bencher, Criterion, Throughput};
use image::Rgb;

fn bench_color_space(c: &mut Criterion) {
    const MAG: u32 = 100_000;
    let mut group = c.benchmark_group("color related stuff");
    group.throughput(Throughput::Elements(MAG.into()));
    let speeds: Vec<f64> = (0..MAG).map(|v| f64::from(v) / f64::from(MAG)).collect();
    let speeds_ref: &[f64] = &speeds;
    group.bench_with_input(
        "palette evaluation",
        speeds_ref,
        |b: &mut Bencher, speeds: &[f64]| {
            b.iter(|| {
                speeds
                    .iter()
                    .map(|s| std::hint::black_box(palette(*s)))
                    .collect::<Vec<_>>()
            })
        },
    );
    drop(speeds_ref);

    let colors: Vec<LinearRGB> = speeds.into_iter().map(palette).collect();
    let colors_ref: &[LinearRGB] = &colors;
    group.bench_with_input(
        "linear<f64> to srgb<u8> conversion",
        colors_ref,
        |b: &mut Bencher, colors: &[LinearRGB]| {
            b.iter(|| {
                colors
                    .iter()
                    .map(|color| std::hint::black_box(Rgb::<u8>::from(*color)))
                    .collect::<Vec<_>>()
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_color_space);
criterion_main!(benches);
