use std::path::PathBuf;
use std::{hint::black_box, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use simd_needle::{MmapFinder, SearchAlgo};
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

fn bench_mmap_small(c: &mut Criterion) {
    let small_data = generate_test_data(1024); // 1KB

    c.bench_function("mmap_small", |b| {
        b.iter(|| {
            let temp_path = "/tmp/small_data.bin";
            std::fs::write(temp_path, &small_data).expect("Failed to write temp file");
            let finder = black_box(
                MmapFinder::new(temp_path, PATTERN.as_bytes().to_vec()).expect("Search failed"),
            );
            finder.find_all(SearchAlgo::Simd).for_each(|pos| {
                let _ = black_box(pos);
            });
            std::fs::remove_file(temp_path).ok();
        });
    });
}

fn bench_mmap_medium(c: &mut Criterion) {
    let medium_data = generate_test_data(1024 * 1024); // 1MB

    c.bench_function("mmap_medium", |b| {
        b.iter(|| {
            let temp_path = "/tmp/medium_data.bin";
            std::fs::write(temp_path, &medium_data).expect("Failed to write temp file");
            let finder = black_box(
                MmapFinder::new(temp_path, PATTERN.as_bytes().to_vec()).expect("Search failed"),
            );
            finder.find_all(SearchAlgo::Simd).for_each(|pos| {
                let _ = black_box(pos);
            });
            std::fs::remove_file(temp_path).ok();
        });
    });
}

fn bench_mmap_large(c: &mut Criterion) {
    let large_data = generate_test_data(10 * 1024 * 1024); // 10MB

    c.bench_function("mmap_large", |b| {
        b.iter(|| {
            let temp_path = "/tmp/large_data.bin";
            std::fs::write(temp_path, &large_data).expect("Failed to write temp file");
            let finder = black_box(
                MmapFinder::new(temp_path, PATTERN.as_bytes().to_vec()).expect("Search failed"),
            );
            finder.find_all(SearchAlgo::Simd).for_each(|pos| {
                let _ = black_box(pos);
            });
            std::fs::remove_file(temp_path).ok();
        });
    });
}

fn bench_mmap_haystacks(c: &mut Criterion) {
    let dir_path = PathBuf::from("./haystacks");
    let walker = WalkDir::new(&dir_path).follow_links(false).into_iter();
    let files: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let index = rand::rng().random_range(0..files.len());
    let selected_file = &files[index];
    eprintln!("Benchmarking with file {:?}", selected_file);

    c.bench_function("mmap_haystacks", |b| {
        b.iter(|| {
            let finder = black_box(
                MmapFinder::new(selected_file, PATTERN.as_bytes().to_vec()).expect("Search failed"),
            );
            finder.find_all(SearchAlgo::Simd).for_each(|pos| {
                let _ = black_box(pos);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_mmap_small,
    bench_mmap_medium,
    bench_mmap_large,
    bench_mmap_haystacks,
);

criterion_main!(benches);
