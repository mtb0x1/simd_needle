/// Boyer-Moore-Horspool search implementation
mod bmh;
/// Naive (brute force) search implementation
mod naive;
/// SIMD-accelerated search implementation
mod simd;

pub use bmh::bmh_search;
pub use naive::naive_search;
pub use simd::simd_search;

/// Supported search algorithms
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Naive,
    Bmh,
    Simd,
}
