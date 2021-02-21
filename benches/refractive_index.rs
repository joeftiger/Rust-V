use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_v::refractive_index::glass;
use rust_v::refractive_index::search_and_lerp;
use std::time::Duration;

fn bench_glass(c: &mut Criterion) {
    let mut group = c.benchmark_group("Refractive Index of Glass");

    group.sample_size(10000);
    group.warm_up_time(Duration::from_secs(10));

    group.bench_function("Search and Get", |b| {
        b.iter(|| search_and_lerp(&glass::INDEX_N, &glass::N, black_box(300.0)))
    });

    // NOTICE: was too slow, so we removed the function.
    // group.bench_function("Calculate", |b| {
    //     b.iter(|| sellmeier_n(black_box(0.3), &glass::INDEX_N, &glass::N))
    // });

    group.bench_function("Calculate Optimized", |b| {
        b.iter(|| glass::sellmeier_n(black_box(0.3)))
    });
}

criterion_group!(benches, bench_glass);
criterion_main!(benches);
