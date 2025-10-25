use std::hint::black_box;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use memchr::memmem;
use rand::Rng;
use simd_needle::{simd_search, Finder, SearchAlgo};
use walkdir::WalkDir;

// Pattern that appears multiple times
const PATTERN: &str = "Hello World!";

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        // Create some repeating patterns to make the search meaningful
        data.push(((i / 64) % 256) as u8);
    }
    data
}

fn bench_search_small(c: &mut Criterion) {
    let small_data = generate_test_data(1024); // 1KB

    let mut group = c.benchmark_group("bench_search_small");

    // Small data benchmarks
    group.bench_function("naive_small", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &small_data[..],
                PATTERN.into(),
                SearchAlgo::Naive,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("bmh_small", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &small_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("simd_small", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &small_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("memchr_small", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&small_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.finish();
}

fn bench_search_medium(c: &mut Criterion) {
    let medium_data = generate_test_data(1024 * 1024); // 1MB

    let mut group = c.benchmark_group("bench_search_medium");

    // Medium data benchmarks
    group.bench_function("naive_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &medium_data[..],
                PATTERN.into(),
                SearchAlgo::Naive,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("bmh_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &medium_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("simd_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &medium_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("memchr_medium", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&medium_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });
    group.finish();
}

fn bench_search_large(c: &mut Criterion) {
    let large_data = generate_test_data(1024 * 1024); // 1MB

    let mut group = c.benchmark_group("bench_search_medium");

    // Large data benchmarks
    group.bench_function("naive_large", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &large_data[..],
                PATTERN.into(),
                SearchAlgo::Naive,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("bmh_large", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &large_data[..],
                PATTERN.into(),
                SearchAlgo::Bmh,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("simd_large", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &large_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("memchr_large", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&large_data, PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.finish();
}

fn bench_search_haystacks(c: &mut Criterion) {
    let dir_path = PathBuf::from("./haystacks");
    //eprintln!("Benchmarking with dir_path {:?} from current path {}", dir_path, std::env::current_dir().unwrap().display());
    let walker = WalkDir::new(&dir_path).follow_links(false).into_iter();
    let files: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    //eprintln!("Benchmarking with files {:?}", files);

    let mut group = c.benchmark_group("bench_search_haystacks");

    // from 0 to files.len() - 1
    let index = rand::rng().random_range(0..files.len());
    let selected_file = &files[index];
    let haystacks_data = std::fs::read(selected_file).expect("Failed to read selected file");

    group.bench_function("simd_haystacks", |b| {
        b.iter(|| {
            let mut finder = black_box(Finder::with_algorithm(
                &haystacks_data[..],
                PATTERN.into(),
                SearchAlgo::Simd,
            ));
            while let Some(Ok(pos)) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("memchr_haystacks", |b| {
        b.iter(|| {
            let mut finder = black_box(memmem::find_iter(&haystacks_data, &PATTERN));
            while let Some(pos) = finder.next() {
                let _ = black_box(pos);
            }
        });
    });

    group.bench_function("memchr_libc_haystacks", |b| {
        b.iter(|| {
            let needle_bytes = PATTERN.as_bytes();
            let n = needle_bytes.len();
            if n == 0 || haystacks_data.len() < n {
                return;
            }
            let mut start = haystacks_data.as_ptr();
            let end = unsafe { start.add(haystacks_data.len()) };

            while start < end {
                let remaining = unsafe { end.offset_from(start) as usize };

                // Search for the first byte of the needle using libc::memchr
                let res = unsafe {
                    libc::memchr(
                        start as *const libc::c_void,
                        needle_bytes[0] as i32,
                        remaining,
                    )
                };
                if res.is_null() {
                    break;
                }

                let pos =
                    unsafe { (res as *const u8).offset_from(haystacks_data.as_ptr()) as usize };

                // Check if the rest of the bytes match the needle
                if pos + n <= haystacks_data.len() && &haystacks_data[pos..pos + n] == needle_bytes
                {
                    let _ = pos;
                }

                // Move one byte forward (allows overlapping matches)
                start = unsafe { (res as *const u8).add(1) };
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_search_haystacks);
//criterion_group!(benches, bench_search_haystacks);
/*criterion_group!(
    benches,
    bench_search_small,
    bench_search_medium,
    bench_search_large,
    bench_search_haystacks
);*/

criterion_main!(benches);
