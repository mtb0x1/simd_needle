use std::hint::black_box;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use libc;
use rand::Rng;
use walkdir::WalkDir;

// Pattern that appears multiple times
const PATTERN: &str = "hello";

fn bench_memchr_libc_haystacks(c: &mut Criterion) {
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

    c.bench_function("memchr_libc_haystacks", |b| {
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
                    black_box(libc::memchr(
                        start as *const libc::c_void,
                        needle_bytes[0] as i32,
                        remaining,
                    ))
                };
                if res.is_null() {
                    break;
                }

                let pos =
                    unsafe { (res as *const u8).offset_from(haystacks_data.as_ptr()) as usize };

                // Check if the rest of the bytes match the needle
                if pos + n <= haystacks_data.len() && &haystacks_data[pos..pos + n] == needle_bytes
                {
                    let _ = black_box(pos);
                }

                // Move one byte forward (allows overlapping matches)
                start = unsafe { (res as *const u8).add(1) };
            }
        });
    });
}

criterion_group!(benches, bench_memchr_libc_haystacks);

criterion_main!(benches);
