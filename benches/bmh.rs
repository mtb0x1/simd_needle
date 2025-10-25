use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use simd_needle::{Finder, FinderTrait, SearchAlgo};

// Pattern that appears multiple times
const PATTERN: &str = "hello";

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        // Create some repeating patterns to make the search meaningful
        data.push(((i / 64) % 256) as u8);
    }
    data
}

fn bench_bmh_small(c: &mut Criterion) {
    let small_data = generate_test_data(1024); // 1KB

    c.bench_function("bmh_small", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &small_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_bmh_medium(c: &mut Criterion) {
    let medium_data = generate_test_data(1024 * 1024); // 1MB

    c.bench_function("bmh_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &medium_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_bmh_large(c: &mut Criterion) {
    let large_data = generate_test_data(10 * 1024 * 1024); // 10MB

    c.bench_function("bmh_large", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &large_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

criterion_group!(benches, bench_bmh_small, bench_bmh_medium, bench_bmh_large,);

criterion_main!(benches);
