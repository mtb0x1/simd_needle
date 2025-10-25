use std::path::PathBuf;
use std::{hint::black_box, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use simd_needle::{Finder, FinderTrait, SearchAlgo};
use walkdir::WalkDir;

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

fn bench_simd_small(c: &mut Criterion) {
    let small_data = generate_test_data(1024); // 1KB

    c.bench_function("simd_small", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &small_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_simd_medium(c: &mut Criterion) {
    let medium_data = generate_test_data(1024 * 1024); // 1MB

    c.bench_function("simd_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &medium_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_simd_large(c: &mut Criterion) {
    let large_data = generate_test_data(10 * 1024 * 1024); // 10MB

    c.bench_function("simd_large", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &large_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ))
            .expect("Search failed");
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_simd_haystacks(c: &mut Criterion) {
    let dir_path = PathBuf::from("./haystacks");
    let walker = WalkDir::new(&dir_path).follow_links(false).into_iter();
    let files: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let index = rand::rng().random_range(0..files.len());
    let selected_file = &files[index];
    let haystacks_data = std::fs::read(selected_file).expect("Failed to read selected file");
    eprintln!("Benchmarking with file {:?}", selected_file);

    c.bench_function("simd_haystacks", |b| {
        b.iter_batched(
            || BufReader::new(&haystacks_data[..]),
            |haystacks_data_reader_for_simd| {
                let mut finder = black_box(Finder::with_algorithm(
                    haystacks_data_reader_for_simd,
                    PATTERN.into(),
                    SearchAlgo::Simd,
                ))
                .expect("Search failed");
                while let Some(Ok(pos)) = finder.next() {
                    let _ = black_box(pos);
                }
            },
            criterion::BatchSize::PerIteration,
        );
    });
}

criterion_group!(
    benches,
    bench_simd_small,
    bench_simd_medium,
    bench_simd_large,
    bench_simd_haystacks,
);

criterion_main!(benches);
