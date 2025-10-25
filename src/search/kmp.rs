#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use tracing::{info, instrument, span, Level};

/// Knuth-Morris-Pratt search implementation.
///
/// Uses a prefix table to efficiently skip comparisons when mismatches occur.
///
/// # Arguments
/// * `haystack` - The data to search in
/// * `needle` - The pattern to search for
///
/// # Returns
/// * `Some(usize)` - Index of the first match
/// * `None` - If no match is found or needle is empty
#[cfg_attr(feature = "debug", instrument(skip(haystack, needle)))]
pub fn kmp_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = haystack.len();
    let m = needle.len();
    if m == 0 || n < m {
        return None;
    }

    #[cfg(feature = "debug")]
    let start_time = Instant::now();

    // Build prefix table (failure function)
    let mut prefix = vec![0; m];
    let mut j = 0;
    for i in 1..m {
        while j > 0 && needle[i] != needle[j] {
            j = prefix[j - 1];
        }
        if needle[i] == needle[j] {
            j += 1;
        }
        prefix[i] = j;
    }

    // Search
    let mut i = 0;
    let mut k = 0;
    while i < n {
        if haystack[i] == needle[k] {
            i += 1;
            k += 1;
            if k == m {
                let pos = i - m;
                #[cfg(feature = "debug")]
                {
                    info!("Match found at position {}", pos);
                    info!(
                        "kmp_search () profiling: total time {:?}",
                        start_time.elapsed()
                    );
                }
                return Some(pos);
            }
        } else if k > 0 {
            k = prefix[k - 1];
        } else {
            i += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_needle() {
        let haystack = b"hello world";
        assert_eq!(kmp_search(haystack, b""), None);
    }

    #[test]
    fn test_needle_longer_than_haystack() {
        let haystack = b"hi";
        let needle = b"hello";
        assert_eq!(kmp_search(haystack, needle), None);
    }

    #[test]
    fn test_no_match() {
        let haystack = b"hello world";
        let needle = b"xyz";
        assert_eq!(kmp_search(haystack, needle), None);
    }

    #[test]
    fn test_match_at_beginning() {
        let haystack = b"hello world";
        let needle = b"hello";
        assert_eq!(kmp_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_match_in_middle() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(kmp_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_match_at_end() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(kmp_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_repeating_pattern() {
        let haystack = b"abababab";
        let needle = b"aba";
        assert_eq!(kmp_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_single_character() {
        let haystack = b"abc";
        let needle = b"b";
        assert_eq!(kmp_search(haystack, needle), Some(1));
    }
}
