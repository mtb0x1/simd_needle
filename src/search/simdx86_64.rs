#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use tracing::{info, instrument, span, Level};

/// SIMD-based search implementation for x86_64 architecture
///
/// # Arguments
/// * `_haystack` - The data to search in
/// * `_needle` - The pattern to search for
///
/// # Returns
/// * `Some(index)` of the first occurrence, or `None` if not found
#[cfg(target_arch = "x86_64")]
#[cfg_attr(feature = "debug", instrument(skip(haystack, needle)))]
pub fn simd_search_x86_64(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;
    // Check if needle is empty or haystack is shorter than needle
    if needle.is_empty() || haystack.len() < needle.len() {
        // Return None immediately since no match is possible
        return None;
    }

    // For needles longer than 16 bytes, fall back to portable SIMD
    if needle.len() > 16 {
        return crate::search::simd_search(haystack, needle);
    }

    #[cfg(feature = "debug")]
    let start_time = Instant::now();

    // This is necessary because we are using x86_64 intrinsics
    unsafe {
        // Load needle into SIMD register (padded with zeros if shorter than 16)
        let mut needle_buf = [0u8; 16];
        needle_buf[..needle.len()].copy_from_slice(needle);
        let needle_simd = _mm_loadu_si128(needle_buf.as_ptr() as *const __m128i);
        let match_mask = (1i32 << needle.len()) - 1;

        let mut i = 0;
        // Loop while there are enough bytes for SIMD comparison
        while i + 16 <= haystack.len() {
            // Prefetch next block for better memory access performance
            if i + 16 <= haystack.len() {
                _mm_prefetch(haystack[i + 16..].as_ptr() as *const i8, _MM_HINT_T0);
            }

            // Load 16 bytes from haystack
            let block = _mm_loadu_si128(haystack[i..].as_ptr() as *const __m128i);
            // Compare the block with needle
            let cmp = _mm_cmpeq_epi8(block, needle_simd);
            // Extract a bitmask where each bit indicates if corresponding bytes are equal
            let mask = _mm_movemask_epi8(cmp);

            // Check if the first needle.len() bytes match (all 1s in the mask for those positions)

            // Verify if the mask indicates a full match for the needle's length
            if (mask & match_mask) == match_mask {
                // Match confirmed, return the starting index
                #[cfg(feature = "debug")]
                {
                    info!("Match found at position {}", i);
                    info!(
                        "simd_search_x86_64 () profiling: total time {:?}",
                        start_time.elapsed()
                    );
                }
                return Some(i);
            }

            i += 1; // Slide by 1 byte for thorough search
        }

        // Fallback for remaining bytes
        while i + needle.len() <= haystack.len() {
            // Use slice starts_with for exact match in remaining bytes
            if haystack[i..].starts_with(needle) {
                // Match found in tail, return index
                #[cfg(feature = "debug")]
                {
                    info!("Match found at position {}", i);
                    info!(
                        "simd_search_x86_64 () profiling: total time {:?}",
                        start_time.elapsed()
                    );
                }
                return Some(i);
            }
            // Increment index for byte-by-byte search
            i += 1;
        }
        // End of fallback search
    }

    // no match was found in the entire haystack
    None
}

#[cfg(test)]
#[cfg(target_arch = "x86_64")]
mod tests {
    use super::*;

    #[test]
    fn test_empty_needle() {
        let haystack = b"hello world";
        assert_eq!(simd_search_x86_64(haystack, b""), None);
    }

    #[test]
    fn test_needle_longer_than_haystack() {
        let haystack = b"hi";
        let needle = b"hello";
        assert_eq!(simd_search_x86_64(haystack, needle), None);
    }

    #[test]
    fn test_no_match() {
        let haystack = b"hello world";
        let needle = b"xyz";
        assert_eq!(simd_search_x86_64(haystack, needle), None);
    }

    #[test]
    fn test_match_at_beginning() {
        let haystack = b"hello world";
        let needle = b"hello";
        assert_eq!(simd_search_x86_64(haystack, needle), Some(0));
    }

    #[test]
    fn test_match_in_middle() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(simd_search_x86_64(haystack, needle), Some(6));
    }

    #[test]
    fn test_match_at_end() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(simd_search_x86_64(haystack, needle), Some(6));
    }

    #[test]
    fn test_repeating_pattern() {
        let haystack = b"abababab";
        let needle = b"aba";
        assert_eq!(simd_search_x86_64(haystack, needle), Some(0));
    }

    #[test]
    fn test_single_character() {
        let haystack = b"abc";
        let needle = b"b";
        assert_eq!(simd_search_x86_64(haystack, needle), Some(1));
    }
}
