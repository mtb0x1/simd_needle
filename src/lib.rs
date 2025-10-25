#![feature(portable_simd)]

//! A fast, streaming needle-in-haystack searcher using various algorithms including SIMD
//!
//! This crate provides utilities for searching byte patterns in streams efficiently,
//! supporting multiple search algorithms including naive, Boyer-Moore-Horspool, and SIMD.
//!
//! Two implementations are provided:
//! - `Finder`: Classic implementation using Vec<u8>

mod finder;
pub mod hex;
mod search;

pub use finder::{Finder, DEFAULT_BUF_SIZE};
pub use search::{bmh_search, naive_search, simd_search, Algorithm as SearchAlgo};

#[cfg(test)]
mod tests;
