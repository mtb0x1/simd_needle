/// Boyer-Moore-Horspool search implementation
mod bmh;
/// Knuth-Morris-Pratt search implementation
mod kmp;
/// Naive (brute force) search implementation
mod naive;
/// SIMD-accelerated search implementation using portable SIMD
mod simd;
/// SIMD-accelerated search implementation for x86_64 architecture
#[cfg(target_arch = "x86_64")]
mod simdx86_64;

pub use bmh::bmh_search;
pub use kmp::kmp_search;
pub use naive::naive_search;
pub use simd::simd_search;
#[cfg(target_arch = "x86_64")]
pub use simdx86_64::simd_search_x86_64;

/// Supported search algorithms
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Naive,
    Bmh,
    Kmp,
    #[cfg(target_arch = "x86_64")]
    SimdX8664,
    Simd,
}
