use core::simd::{cmp::SimdPartialEq, LaneCount, Simd, SupportedLaneCount};

#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use tracing::{info, instrument, span, Level};

// Select optimal SIMD width based on target architecture
#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
const SIMD_LANES: usize = 64;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
const SIMD_LANES: usize = 32;

#[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
const SIMD_LANES: usize = 16;

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
const SIMD_LANES: usize = 16;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
const SIMD_LANES: usize = 16;

// long story short, we use hardware SIMD size and overload it
// most likely the botelneck is on computation not on memory access
// maybe, who knows
// FIXME: test different sizes per different architectures
const SIMD_BOOST: usize = 4;
const SIMD_SIZE_BOOSTED: usize = (SIMD_LANES * SIMD_BOOST).min(128);

/// SIMD scan helper that searches for the first byte of needle in haystack
///
/// Returns the index of a potential match candidate
fn simd_scan_first_byte<const N: usize>(haystack: &[u8], first_byte: u8) -> Option<usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    let needle_simd = Simd::<u8, N>::splat(first_byte);
    let mut i = 0;

    while i + N <= haystack.len() {
        // Prefetch next chunk for better memory access performance
        if i + N + N <= haystack.len() {
            core::intrinsics::prefetch_read_data::<u8, 3>(&haystack[i + N]);
        }

        let chunk = Simd::<u8, N>::from_slice(&haystack[i..i + N]);
        let matches = chunk.simd_eq(needle_simd);
        let mask = matches.to_bitmask();

        if mask != 0 {
            // Found at least one matching byte, find first one
            let offset = mask.trailing_zeros() as usize;
            return Some(i + offset);
        }

        i += N;
    }

    // Check remaining bytes
    haystack[i..]
        .iter()
        .position(|&b| b == first_byte)
        .map(|pos| i + pos)
}

/// SIMD-based search implementation using portable SIMD
///
/// Uses a two-step approach:
/// 1. SIMD scan to find candidates matching the first byte
/// 2. Verification of full needle match at candidate positions
#[cfg_attr(feature = "debug", instrument(skip(haystack, needle)))]
pub fn simd_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    // Single byte needle - use SIMD scan directly
    if needle.len() == 1 {
        return simd_scan_first_byte::<SIMD_SIZE_BOOSTED>(haystack, needle[0]);
    }

    let first_byte = needle[0];
    let mut search_start = 0;

    #[cfg(feature = "debug")]
    let search_span = span!(Level::INFO, "search_loop").entered();

    while search_start + needle.len() <= haystack.len() {
        #[cfg(feature = "debug")]
        let start_time = Instant::now();
        // Use SIMD to find next candidate position
        match simd_scan_first_byte::<SIMD_SIZE_BOOSTED>(&haystack[search_start..], first_byte) {
            Some(offset) => {
                let candidate_pos = search_start + offset;

                // Check if we have enough bytes remaining
                if candidate_pos + needle.len() > haystack.len() {
                    return None;
                }

                // Verify full match
                if &haystack[candidate_pos..candidate_pos + needle.len()] == needle {
                    #[cfg(feature = "debug")]
                    {
                        info!("Match found at position {}", candidate_pos);

                        info!(
                            "simd_search () profiling: total time {:?}",
                            start_time.elapsed()
                        );
                    }
                    return Some(candidate_pos);
                }

                // Move past this candidate
                search_start = candidate_pos + 1;
            }
            None => return None,
        }
    }

    #[cfg(feature = "debug")]
    drop(search_span);

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_needle() {
        let haystack = b"hello world";
        assert_eq!(simd_search(haystack, b""), None);
    }

    #[test]
    fn test_needle_longer_than_haystack() {
        let haystack = b"hi";
        let needle = b"hello";
        assert_eq!(simd_search(haystack, needle), None);
    }

    #[test]
    fn test_no_match() {
        let haystack = b"hello world";
        let needle = b"xyz";
        assert_eq!(simd_search(haystack, needle), None);
    }

    #[test]
    fn test_match_at_beginning() {
        let haystack = b"hello world";
        let needle = b"hello";
        assert_eq!(simd_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_match_in_middle() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(simd_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_match_at_end() {
        let haystack = b"hello world";
        let needle = b"ld";
        assert_eq!(simd_search(haystack, needle), Some(9));
    }

    #[test]
    fn test_repeating_pattern() {
        let haystack = b"abababab";
        let needle = b"aba";
        assert_eq!(simd_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_single_character() {
        let haystack = b"abc";
        let needle = b"b";
        assert_eq!(simd_search(haystack, needle), Some(1));
    }

    #[test]
    fn test_scan_first_byte() {
        let haystack = b"hello world";
        assert_eq!(simd_scan_first_byte::<SIMD_LANES>(haystack, b'w'), Some(6));
        assert_eq!(simd_scan_first_byte::<SIMD_LANES>(haystack, b'z'), None);
    }
}
