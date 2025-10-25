#![feature(portable_simd)]
#![feature(const_cmp)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]

//! A fast, streaming needle-in-haystack searcher using various algorithms including SIMD
//!
//! This crate provides utilities for searching byte patterns in streams efficiently,
//! supporting multiple search algorithms including naive, Boyer-Moore-Horspool, KMP, and SIMD.
//!
//! Three implementations are provided:
//! - `Finder`: Streaming implementation for any Read source
//! - `MmapFinder`: Zero-copy implementation for memory-mapped files
mod finder;
pub mod hex;
mod mmap_finder;
mod search;

pub use finder::{Finder, FinderError, FinderTrait, DEFAULT_BUF_SIZE};
pub use mmap_finder::{find_in_file, find_in_mmap, MmapFinder, MmapFinderError};
#[cfg(target_arch = "x86_64")]
pub use search::simd_search_x86_64;
pub use search::{bmh_search, kmp_search, naive_search, simd_search, Algorithm as SearchAlgo};

#[cfg(test)]
mod tests;
