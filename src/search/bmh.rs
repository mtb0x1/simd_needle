#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use tracing::{info, instrument, span, Level};

/// Boyer-Moore-Horspool search (simple implementation).
///
/// # Arguments
/// * `haystack` - The data to search in
/// * `needle` - The pattern to search for
///
/// # Returns
/// * `Some(usize)` - Index of the first match
/// * `None` - If no match is found or needle is empty
#[cfg_attr(feature = "debug", instrument(skip(haystack, needle)))]
pub fn bmh_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let m = needle.len();
    if m == 0 || haystack.len() < m {
        return None;
    }

    #[cfg(feature = "debug")]
    let start_time = Instant::now();

    // Build bad-character shift table
    let mut shift = [m; 256usize];
    for i in 0..m - 1 {
        shift[needle[i] as usize] = m - 1 - i;
    }

    let mut i = 0usize;
    while i + m <= haystack.len() {
        let mut j = (m - 1) as isize;
        while j >= 0 && haystack[i + j as usize] == needle[j as usize] {
            j -= 1;
        }
        if j < 0 {
            #[cfg(feature = "debug")]
            {
                info!("Match found at position {}", i);
                info!(
                    "bmh_search () profiling: total time {:?}",
                    start_time.elapsed()
                );
            }
            return Some(i);
        }
        let next_byte = haystack[i + m - 1];
        i += shift[next_byte as usize];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_needle() {
        let haystack = b"hello world";
        assert_eq!(bmh_search(haystack, b""), None);
    }

    #[test]
    fn test_needle_longer_than_haystack() {
        let haystack = b"hi";
        let needle = b"hello";
        assert_eq!(bmh_search(haystack, needle), None);
    }

    #[test]
    fn test_no_match() {
        let haystack = b"hello world";
        let needle = b"xyz";
        assert_eq!(bmh_search(haystack, needle), None);
    }

    #[test]
    fn test_match_at_beginning() {
        let haystack = b"hello world";
        let needle = b"hello";
        assert_eq!(bmh_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_match_in_middle() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(bmh_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_match_at_end() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(bmh_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_repeating_pattern() {
        let haystack = b"abababab";
        let needle = b"aba";
        assert_eq!(bmh_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_single_character() {
        let haystack = b"abc";
        let needle = b"b";
        assert_eq!(bmh_search(haystack, needle), Some(1));
    }
}
