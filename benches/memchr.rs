use std::hint::black_box;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use memchr::memmem;
use rand::Rng;
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

fn bench_memchr_small(c: &mut Criterion) {
    let small_data = generate_test_data(1024); // 1KB

    c.bench_function("memchr_small", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&small_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_memchr_medium(c: &mut Criterion) {
    let medium_data = generate_test_data(1024 * 1024); // 1MB

    c.bench_function("memchr_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&medium_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_memchr_large(c: &mut Criterion) {
    let large_data = generate_test_data(10 * 1024 * 1024); // 10MB

    c.bench_function("memchr_large", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&large_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

fn bench_memchr_haystacks(c: &mut Criterion) {
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

    c.bench_function("memchr_haystacks", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&haystacks_data, &PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_memchr_small,
    bench_memchr_medium,
    bench_memchr_large,
    bench_memchr_haystacks,
);

criterion_main!(benches);
