#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use tracing::{info, instrument, span, Level};

/// A simple, naive byte-slice search that returns the index of the first
/// occurrence of `needle` inside `haystack`, or `None` if not found.
///
/// This mirrors the previous behaviour using `.windows(...).position(...)`
/// but is extracted so the algorithm can be named and replaced later.
#[cfg_attr(feature = "debug", instrument(skip(haystack, needle)))]
pub fn naive_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    #[cfg(feature = "debug")]
    let start_time = Instant::now();

    let last_start = haystack.len() - needle.len();
    let result = (0..=last_start).find(|&i| &haystack[i..i + needle.len()] == needle);

    #[cfg(feature = "debug")]
    if let Some(pos) = result {
        info!("Match found at position {}", pos);
        info!(
            "naive_search () profiling: total time {:?}",
            start_time.elapsed()
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_needle() {
        let haystack = b"hello world";
        assert_eq!(naive_search(haystack, b""), None);
    }

    #[test]
    fn test_needle_longer_than_haystack() {
        let haystack = b"hi";
        let needle = b"hello";
        assert_eq!(naive_search(haystack, needle), None);
    }

    #[test]
    fn test_no_match() {
        let haystack = b"hello world";
        let needle = b"xyz";
        assert_eq!(naive_search(haystack, needle), None);
    }

    #[test]
    fn test_match_at_beginning() {
        let haystack = b"hello world";
        let needle = b"hello";
        assert_eq!(naive_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_match_in_middle() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(naive_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_match_at_end() {
        let haystack = b"hello world";
        let needle = b"world";
        assert_eq!(naive_search(haystack, needle), Some(6));
    }

    #[test]
    fn test_repeating_pattern() {
        let haystack = b"abababab";
        let needle = b"aba";
        assert_eq!(naive_search(haystack, needle), Some(0));
    }

    #[test]
    fn test_single_character() {
        let haystack = b"abc";
        let needle = b"b";
        assert_eq!(naive_search(haystack, needle), Some(1));
    }
}
