#[cfg(test)]
mod tests {
    use crate::{Finder, DEFAULT_BUF_SIZE};
    use std::io::Cursor;

    fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
        let finder = Finder::new(Cursor::new(haystack), needle.to_vec(), None);
        finder.map(|r| r.unwrap()).collect()
    }

    #[test]
    fn test_simple_find() {
        assert_eq!(find_all(b"this is a test string", b"test"), vec![10]);
    }

    #[test]
    fn test_no_match() {
        assert_eq!(
            find_all(b"this is a test string", b"nothere"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn test_match_at_beginning() {
        assert_eq!(find_all(b"test string", b"test"), vec![0]);
    }

    #[test]
    fn test_match_at_end() {
        assert_eq!(find_all(b"string test", b"test"), vec![7]);
    }

    #[test]
    fn test_multiple_matches() {
        assert_eq!(find_all(b"test test test", b"test"), vec![0, 5, 10]);
    }

    #[test]
    fn test_overlapping_matches() {
        assert_eq!(find_all(b"ababab", b"abab"), vec![0, 2]);
        assert_eq!(find_all(b"aaaaa", b"aa"), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_empty_haystack() {
        assert_eq!(find_all(b"", b"test"), Vec::<usize>::new());
    }

    #[test]
    fn test_empty_needle() {
        let finder = Finder::new(Cursor::new(&b"test"[..]), vec![], None);
        assert!(finder.map(|r| r.unwrap()).collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_exact_match() {
        assert_eq!(find_all(b"test", b"test"), vec![0]);
    }

    #[test]
    fn test_match_across_buffer_boundary() {
        let mut haystack = Vec::new();
        haystack.extend_from_slice(&[0; DEFAULT_BUF_SIZE - 2]);
        haystack.extend_from_slice(b"needle");
        let finder = Finder::new(Cursor::new(haystack), b"needle".to_vec(), None);
        let results: Vec<_> = finder.map(|r| r.unwrap()).collect();
        assert_eq!(results, vec![DEFAULT_BUF_SIZE - 2]);
    }

    #[test]
    fn test_needle_longer_than_buffer() {
        let needle = vec![b'a'; DEFAULT_BUF_SIZE * 2];
        let mut haystack = vec![b'b'; 10];
        haystack.extend_from_slice(&needle);
        let finder = Finder::with_buffer_size(
            Cursor::new(haystack),
            needle.clone(),
            DEFAULT_BUF_SIZE,
            None,
        );
        let results: Vec<_> = finder.map(|r| r.unwrap()).collect();
        assert_eq!(results, vec![10]);
    }

    #[test]
    fn test_per_thread_limit_calculation() {
        // Replicate the logic from main.rs for a variety of memory limits and thread counts
        let max_threads = 8; // pretend Rayon has 8 threads

        let cases = vec![0usize, 1, 2, 7, 8, 9, 100, 1_000_000_000];
        for memory_limit in cases {
            let threads_to_use = if memory_limit == 0 {
                1
            } else if memory_limit < max_threads {
                std::cmp::max(1, memory_limit)
            } else {
                max_threads
            };

            let mut per_thread_limit = if threads_to_use > 0 {
                memory_limit / threads_to_use
            } else {
                memory_limit
            };
            if per_thread_limit == 0 {
                per_thread_limit = 1;
            }

            // Check invariant: per_thread_limit >= 1
            assert!(per_thread_limit >= 1);

            // Check that total <= memory_limit when memory_limit >= threads_to_use
            if memory_limit >= threads_to_use {
                assert!(per_thread_limit.saturating_mul(threads_to_use) <= memory_limit);
            }
        }
    }
}
