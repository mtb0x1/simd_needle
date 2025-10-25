# simd_needle

Fast Rust library to search for byte patterns in files using SIMD acceleration. Supports multiple search algorithms and parallel processing.

## Features

- SIMD-accelerated pattern matching for high performance
- Multiple search algorithms: naive, Boyer-Moore-Horspool (BMH), Knuth-Morris-Pratt (KMP), SIMD-based
- Parallel processing for searching multiple files
- Hex string support for binary patterns
- Flexible algorithm mapping by file type
- Memory-efficient with configurable limits
- **Zero-copy APIs for memory-mapped files** - efficient access to large datasets without copying data

## Library Usage

The crate provides both streaming and zero-copy APIs for different use cases:

### Streaming Search (for any Read source)

```rust
use std::io::Cursor;
use simd_needle::{Finder, SearchAlgo};

let haystack = b"hello world";
let needle = b"world";

let finder = Finder::new(Cursor::new(haystack), needle.to_vec(), Some(SearchAlgo::Naive)).unwrap();
let positions: Vec<usize> = finder.map(|r| r.unwrap()).collect();
assert_eq!(positions, vec![6]);
```

### Zero-copy Search (for memory-mapped files)

```rust
use simd_needle::{MmapFinder, SearchAlgo};

let finder = MmapFinder::new("large_file.txt", b"pattern".to_vec()).unwrap();

// Find first occurrence
if let Some(pos) = finder.find_first(SearchAlgo::Simd) {
    println!("Found at position: {}", pos);
}

// Find all occurrences
let positions: Vec<usize> = finder.find_all(SearchAlgo::Simd).collect();

// Convenience function
let positions = simd_needle::find_in_file("large_file.txt", b"pattern", SearchAlgo::Simd).unwrap();
```

## Installation

Add this to your `Cargo.toml`:

```toml
simd_needle = { git = "https://github.com/mtb0x1/simd_needle.git" }
```

## Algorithms

- **naive**: Simple linear search
- **bmh**: Boyer-Moore-Horspool algorithm, efficient for most patterns
- **kmp**: Knuth-Morris-Pratt algorithm, good for repetitive patterns
- **simd**: SIMD-accelerated search using CPU vector instructions

## Benchmarks

Benchmarks are available in the `benches/` directory. Run with:

```bash
cargo bench
```

```bash
bash benches/bench_cli.sh
```

*Benchmarks are work in progress.*

```Running on x86_64 architecture, including simdx86_64 benchmark.
Benchmark 1: memchr crate with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
  Time (mean ± σ):     142.4 ms ±  23.5 ms    [User: 109.6 ms, System: 34.0 ms]
  Range (min … max):   103.4 ms … 175.8 ms    22 runs
 
Benchmark 2: memchr_libc with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
  Time (mean ± σ):     133.1 ms ±  24.3 ms    [User: 102.6 ms, System: 31.5 ms]
  Range (min … max):    98.6 ms … 169.0 ms    21 runs
 
Benchmark 3: simdx86_64 with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
  Time (mean ± σ):     138.4 ms ±  23.9 ms    [User: 108.1 ms, System: 31.3 ms]
  Range (min … max):    97.7 ms … 179.0 ms    19 runs
 
Benchmark 4: simd with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
  Time (mean ± σ):     139.0 ms ±  24.1 ms    [User: 107.6 ms, System: 32.4 ms]
  Range (min … max):   104.3 ms … 176.6 ms    18 runs
 
Benchmark 5: simd_mmap_finder with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
  Time (mean ± σ):     135.9 ms ±  20.9 ms    [User: 100.5 ms, System: 36.6 ms]
  Range (min … max):   105.8 ms … 165.4 ms    19 runs
 
Summary
  memchr_libc with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello' ran
    1.02 ± 0.24 times faster than simd_mmap_finder with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
    1.04 ± 0.26 times faster than simdx86_64 with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
    1.04 ± 0.26 times faster than simd with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
    1.07 ± 0.26 times faster than memchr crate with --haystack='haystacks/opensubtitles/en-huge.txt' --needle='hello'
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.