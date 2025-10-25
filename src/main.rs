//! Command-line interface for the simd_needle searcher
//!
//! Provides a fast, multi-threaded file search utility supporting multiple
//! search algorithms and pattern matching.

use glob::Pattern;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use walkdir::WalkDir;

use clap::{builder::TypedValueParser, Parser};
use rayon::prelude::*;
use simd_needle::{hex, Finder, SearchAlgo};

/// A list of search algorithms to be used in sequence
#[derive(Clone, Debug)]
struct AlgoList(Vec<SearchAlgo>);

impl FromStr for AlgoList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "naive" => out.push(SearchAlgo::Naive),
                "bmh" => out.push(SearchAlgo::Bmh),
                other => return Err(format!("Unknown search algorithm: {}", other)),
            }
        }
        if out.is_empty() {
            out.push(SearchAlgo::Naive);
        }
        Ok(AlgoList(out))
    }
}

#[derive(Clone)]
struct AlgoListParser;

impl TypedValueParser for AlgoListParser {
    type Value = AlgoList;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value.to_str().ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidUtf8,
                "Algo list must be valid UTF-8",
            )
        })?;
        AlgoList::from_str(s).map_err(|e| clap::Error::raw(clap::error::ErrorKind::InvalidValue, e))
    }
}

#[derive(Clone, Debug)]
struct AlgoMap(HashMap<Pattern, SearchAlgo>);

impl AlgoMap {
    fn get_for_path(&self, path: &str) -> Option<SearchAlgo> {
        self.0
            .iter()
            .find(|(pattern, _)| pattern.matches(path))
            .map(|(_, algo)| *algo)
    }
}

impl FromStr for AlgoMap {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = HashMap::new();
        for part in s.split(',') {
            let (pattern, algo) = part
                .split_once('=')
                .ok_or_else(|| format!("Invalid mapping (expected PATTERN=ALGO): {}", part))?;

            let pattern =
                Pattern::new(pattern.trim()).map_err(|e| format!("Invalid glob pattern: {}", e))?;

            let algo = match algo.trim().to_lowercase().as_str() {
                "naive" => SearchAlgo::Naive,
                "bmh" => SearchAlgo::Bmh,
                "simd" => SearchAlgo::Simd,
                other => return Err(format!("Unknown search algorithm: {}", other)),
            };

            map.insert(pattern, algo);
        }
        Ok(AlgoMap(map))
    }
}

#[derive(Clone)]
struct AlgoMapParser;

impl TypedValueParser for AlgoMapParser {
    type Value = AlgoMap;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value.to_str().ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidUtf8,
                "Algo map must be valid UTF-8",
            )
        })?;
        AlgoMap::from_str(s).map_err(|e| clap::Error::raw(clap::error::ErrorKind::InvalidValue, e))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The needle to search for
    needle: String,

    /// The haystack file to search in (mutually exclusive with --haystacks-dir)
    #[arg(required_unless_present = "haystacks_dir")]
    haystack: Option<String>,

    /// A directory containing multiple haystack files to search in parallel
    #[arg(long = "haystacks-dir", value_name = "DIR")]
    haystacks_dir: Option<String>,

    /// Maximum directory recursion depth when scanning haystacks-dir (0 = only that dir). Defaults to unlimited.
    #[arg(long = "max-depth")]
    max_depth: Option<usize>,

    /// Treat the needle as a hex string
    #[arg(short = 'x', long)]
    hex: bool,

    /// The memory limit in bytes
    #[arg(default_value_t = 1_000_000_000)]
    memory_limit: usize,

    /// Comma-separated list of search algorithms to assign to worker threads.
    /// Examples: "naive" (all threads), "bmh" (all threads), or "naive,bmh" to
    /// alternate/assign algorithms by thread index (modulo the list length).
    #[arg(long = "algos", value_parser = AlgoListParser)]
    algos: Option<AlgoList>,

    /// Comma-separated mappings of glob patterns to algorithms.
    /// Example: "*.log=bmh,*.bin=naive" to use BMH for .log files and naive for .bin files.
    /// Falls back to --algos if no pattern matches.
    #[arg(long = "algo-map", value_parser = AlgoMapParser)]
    algo_map: Option<AlgoMap>,
}

fn main() {
    let args = Args::parse();

    let needle = if args.hex {
        match hex::decode(args.needle) {
            Ok(needle) => needle,
            Err(e) => {
                eprintln!("Error decoding hex needle: {}", e);
                process::exit(1);
            }
        }
    } else {
        args.needle.into_bytes()
    };

    // Parse algorithm choices
    let algos = args
        .algos
        .map(|a| a.0)
        .unwrap_or_else(|| vec![SearchAlgo::Naive]);

    // Compute how many threads to use so the sum of per-thread buffers can be kept <= memory_limit.
    // If the configured memory_limit is smaller than the available Rayon threads, reduce the
    // number of threads so each thread can get at least 1 byte (or a meaningful share).
    let max_threads = rayon::current_num_threads();
    let threads_to_use = if args.memory_limit == 0 {
        1
    } else if args.memory_limit < max_threads {
        // Use at most memory_limit threads (so each can have >= 1 byte). This keeps
        // per_thread_limit = memory_limit / threads_to_use >= 1.
        std::cmp::max(1, args.memory_limit)
    } else {
        max_threads
    };

    // Now compute per-thread buffer limit (floor division). This guarantees
    // per_thread_limit * threads_to_use <= args.memory_limit when args.memory_limit >= threads_to_use.
    // For args.memory_limit == 0 we fall back to 1 to keep buffers functional.
    let mut per_thread_limit = if threads_to_use > 0 {
        args.memory_limit / threads_to_use
    } else {
        args.memory_limit
    };
    if per_thread_limit == 0 {
        per_thread_limit = 1;
    }

    // If a directory of haystacks was provided, process files in parallel.
    if let Some(dir) = args.haystacks_dir {
        let dir_path = PathBuf::from(dir);
        let walker = WalkDir::new(&dir_path).follow_links(false).into_iter();
        let entries: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(max) = args.max_depth {
                    // WalkDir depth is root=0, so compute relative depth
                    if let Ok(rel) = e.path().strip_prefix(&dir_path) {
                        return rel.components().count() <= max.saturating_add(1);
                    }
                }
                true
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // If we reduced thread count, create a local Rayon thread pool with the desired size so we
        // actually limit concurrency and thus total memory usage.
        if threads_to_use != max_threads {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads_to_use)
                .build()
                .expect("failed to build thread pool");
            pool.install(|| {
                entries.par_iter().for_each(|path| match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        let thread_index = rayon::current_thread_index().unwrap_or(0);

                        // Pick algorithm based on filename pattern or fallback to thread-based assignment
                        let chosen_algo = if let Some(ref map) = args.algo_map {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                map.get_for_path(name)
                                    .unwrap_or_else(|| algos[thread_index % algos.len()])
                            } else {
                                algos[thread_index % algos.len()]
                            }
                        } else {
                            algos[thread_index % algos.len()]
                        };

                        let finder = Finder::with_buffer_size(
                            reader,
                            needle.clone(),
                            per_thread_limit,
                            Some(chosen_algo),
                        );
                        for result in finder {
                            match result {
                                Ok(offset) => println!("{}:{}", path.display(), offset),
                                Err(e) => eprintln!("Error reading {}: {}", path.display(), e),
                            }
                        }
                    }
                    Err(e) => eprintln!("Error opening {}: {}", path.display(), e),
                });
            });
        } else {
            entries.par_iter().for_each(|path| match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let thread_index = rayon::current_thread_index().unwrap_or(0);

                    // Pick algorithm based on filename pattern or fallback to thread-based assignment
                    let chosen_algo = if let Some(ref map) = args.algo_map {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            map.get_for_path(name)
                                .unwrap_or_else(|| algos[thread_index % algos.len()])
                        } else {
                            algos[thread_index % algos.len()]
                        }
                    } else {
                        algos[thread_index % algos.len()]
                    };

                    let finder = Finder::with_buffer_size(
                        reader,
                        needle.clone(),
                        per_thread_limit,
                        Some(chosen_algo),
                    );
                    for result in finder {
                        match result {
                            Ok(offset) => println!("{}:{}", path.display(), offset),
                            Err(e) => eprintln!("Error reading {}: {}", path.display(), e),
                        }
                    }
                }
                Err(e) => eprintln!("Error opening {}: {}", path.display(), e),
            });
        }
        return;
    }

    // Otherwise, open the single haystack file provided.
    let haystack_path = args.haystack.expect("haystack file argument required");
    let file = match File::open(&haystack_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening haystack file: {}", e);
            process::exit(1);
        }
    };

    let reader = BufReader::new(file);

    // Pick algorithm based on filename pattern or fallback to first algorithm in list
    let chosen_algo = if let Some(ref map) = args.algo_map {
        if let Some(name) = PathBuf::from(&haystack_path)
            .file_name()
            .and_then(|n| n.to_str())
        {
            map.get_for_path(name).unwrap_or(algos[0])
        } else {
            algos[0]
        }
    } else {
        algos[0]
    };

    // For single-file runs, cap the buffer to the full memory_limit (single thread uses all allowed memory)
    let finder = Finder::with_buffer_size(reader, needle, args.memory_limit, Some(chosen_algo));

    for result in finder {
        match result {
            Ok(offset) => println!("{}", offset),
            Err(e) => {
                eprintln!("Error reading haystack: {}", e);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Args {
        use clap::Parser;
        Args::try_parse_from(args).unwrap()
    }

    #[test]
    fn test_parse_valid_algos() {
        let args = parse_args(&["finder", "needle", "file", "--algos", "naive,bmh"]);
        assert!(args.algos.is_some());
        let algos = args.algos.unwrap();
        assert_eq!(algos.0, &[SearchAlgo::Naive, SearchAlgo::Bmh]);
    }

    #[test]
    #[should_panic(expected = "Unknown search algorithm")]
    fn test_parse_invalid_algo() {
        parse_args(&["finder", "needle", "file", "--algos", "fast"]);
    }

    #[test]
    fn test_parse_valid_algo_map() {
        let args = parse_args(&[
            "finder",
            "needle",
            "file",
            "--algo-map",
            "*.txt=naive,*.bin=bmh",
        ]);
        assert!(args.algo_map.is_some());
        let map = args.algo_map.unwrap();
        assert!(map.get_for_path("test.txt") == Some(SearchAlgo::Naive));
        assert!(map.get_for_path("data.bin") == Some(SearchAlgo::Bmh));
    }

    #[test]
    #[should_panic(expected = "Invalid mapping")]
    fn test_parse_invalid_algo_map_format() {
        parse_args(&["finder", "needle", "file", "--algo-map", "*.txt:naive"]);
    }

    #[test]
    #[should_panic(expected = "Invalid glob pattern")]
    fn test_parse_invalid_glob_pattern() {
        parse_args(&["finder", "needle", "file", "--algo-map", "[=naive"]);
    }
}
