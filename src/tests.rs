#[cfg(test)]
mod tests {
    use crate::{search::Algorithm, Finder, FinderTrait, DEFAULT_BUF_SIZE};
    use proptest::prelude::*;
    use std::io::Cursor;

    fn find_all(haystack: &[u8], needle: &[u8], algo: Algorithm) -> Vec<usize> {
        let finder = Finder::new(Cursor::new(haystack), needle.to_vec(), Some(algo)).unwrap();
        finder.map(|r| r.unwrap()).collect()
    }

    macro_rules! test_all_algos {
        ($test_name:ident, $test_body:expr) => {
            paste::paste! {
                #[test]
                fn [<$test_name _naive>]() {
                    let algo = Algorithm::Naive;
                    $test_body(algo);
                }

                #[test]
                fn [<$test_name _bmh>]() {
                    let algo = Algorithm::Bmh;
                    $test_body(algo);
                }

                #[test]
                fn [<$test_name _kmp>]() {
                    let algo = Algorithm::Kmp;
                    $test_body(algo);
                }

                #[test]
                fn [<$test_name _simdx8664>]() {
                    #[cfg(target_arch = "x86_64")]{
                        let algo = Algorithm::SimdX8664;
                        $test_body(algo);
                    }
                }


                #[test]
                fn [<$test_name _simd>]() {
                    let algo = Algorithm::Simd;
                    $test_body(algo);
                }
            }
        };
    }

    test_all_algos!(test_simple_find, |algo: Algorithm| {
        assert_eq!(find_all(b"this is a test string", b"test", algo), vec![10]);
    });

    test_all_algos!(test_no_match, |algo: Algorithm| {
        assert_eq!(
            find_all(b"this is a test string", b"nothere", algo),
            Vec::<usize>::new()
        );
    });

    test_all_algos!(test_match_at_beginning, |algo: Algorithm| {
        assert_eq!(find_all(b"test string", b"test", algo), vec![0]);
    });

    test_all_algos!(test_match_at_end, |algo: Algorithm| {
        assert_eq!(find_all(b"string test", b"test", algo), vec![7]);
    });

    test_all_algos!(test_multiple_matches, |algo: Algorithm| {
        assert_eq!(find_all(b"test test test", b"test", algo), vec![0, 5, 10]);
    });

    test_all_algos!(test_overlapping_matches, |algo: Algorithm| {
        assert_eq!(find_all(b"ababab", b"abab", algo), vec![0, 2]);
        assert_eq!(find_all(b"aaaaa", b"aa", algo), vec![0, 1, 2, 3]);
    });

    test_all_algos!(test_empty_haystack, |algo: Algorithm| {
        assert_eq!(find_all(b"", b"test", algo), Vec::<usize>::new());
    });

    test_all_algos!(test_exact_match, |algo: Algorithm| {
        assert_eq!(find_all(b"test", b"test", algo), vec![0]);
    });

    test_all_algos!(test_match_across_buffer_boundary, |algo: Algorithm| {
        let mut haystack = Vec::new();
        haystack.extend_from_slice(&[0; DEFAULT_BUF_SIZE - 2]);
        haystack.extend_from_slice(b"needle");
        let finder = Finder::new(Cursor::new(haystack), b"needle".to_vec(), Some(algo)).unwrap();
        let results: Vec<_> = finder.map(|r| r.unwrap()).collect();
        assert_eq!(results, vec![DEFAULT_BUF_SIZE - 2]);
    });

    test_all_algos!(test_needle_longer_than_buffer, |algo: Algorithm| {
        let needle = vec![b'a'; DEFAULT_BUF_SIZE * 2];
        let mut haystack = vec![b'b'; 10];
        haystack.extend_from_slice(&needle);
        let result = Finder::with_buffer_size(
            Cursor::new(haystack),
            needle.clone(),
            DEFAULT_BUF_SIZE,
            Some(algo),
        );
        assert!(result.is_err());
    });

    #[test]
    fn test_empty_needle_naive() {
        let algo = Algorithm::Naive;
        let result = Finder::new(Cursor::new(&b"test"[..]), vec![], Some(algo));
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_needle_bmh() {
        let algo = Algorithm::Bmh;
        let result = Finder::new(Cursor::new(&b"test"[..]), vec![], Some(algo));
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_needle_kmp() {
        let algo = Algorithm::Kmp;
        let result = Finder::new(Cursor::new(&b"test"[..]), vec![], Some(algo));
        assert!(result.is_err());
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

    proptest! {
        #[test]
        fn all_algorithms_return_identical_results(
            haystack in prop::collection::vec(any::<u8>(), 0..1000),
            needle in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            // Ensure needle is not longer than DEFAULT_BUF_SIZE
            prop_assume!(needle.len() <= DEFAULT_BUF_SIZE);

            // Collect results from all algorithms
            let naive_result = find_all(&haystack, &needle, Algorithm::Naive);
            let bmh_result = find_all(&haystack, &needle, Algorithm::Bmh);
            let kmp_result = find_all(&haystack, &needle, Algorithm::Kmp);
            let simd_result = find_all(&haystack, &needle, Algorithm::Simd);
            #[cfg(target_arch = "x86_64")]
            let simdx86_64_result = find_all(&haystack, &needle, Algorithm::SimdX8664);

            // All results should be identical
            prop_assert_eq!(&naive_result, &bmh_result);
            prop_assert_eq!(&naive_result, &kmp_result);
            prop_assert_eq!(&naive_result, &simd_result);
            #[cfg(target_arch = "x86_64")]
            prop_assert_eq!(&naive_result, &simdx86_64_result);
        }
    }

    #[test]
    fn test_mmap_finder() {
        use crate::MmapFinder;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let haystack = b"hello world hello universe";
        let needle = b"hello";

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(haystack).unwrap();
        temp_file.flush().unwrap();

        // Test MmapFinder
        let finder = MmapFinder::new(temp_file.path(), needle.to_vec()).unwrap();

        // Test find_first
        let first_pos = finder.find_first(Algorithm::Naive);
        assert_eq!(first_pos, Some(0));

        // Test find_all
        let positions: Vec<usize> = finder.find_all(Algorithm::Naive).collect();
        assert_eq!(positions, vec![0, 12]);

        // Test convenience function
        let positions2 = crate::find_in_file(temp_file.path(), needle, Algorithm::Naive).unwrap();
        assert_eq!(positions2, vec![0, 12]);
    }
}
