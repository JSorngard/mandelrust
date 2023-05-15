use criterion::{criterion_group, criterion_main, Criterion};
use color_space::palette;

fn bench_color_curves(c: &mut Criterion) {
    const MAG: u32 = 100000;
    let speeds: Vec<f64> = (0..MAG).map(|v|f64::from(v) / f64::from(MAG)).collect();
    c.bench_function("color curves", |b| b.iter(|| speeds.iter().map(|s| std::hint::black_box(palette(*s)))));
}

criterion_group!(benches, bench_color_curves);
criterion_main!(benches);