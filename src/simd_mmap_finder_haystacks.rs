use clap::Parser;
use simd_needle::{MmapFinder, SearchAlgo};
use std::fs::File;
use std::io::BufReader;
use std::path;

#[cfg(feature = "debug")]
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The haystack string to search in
    haystack: String,

    /// The needle string to search for
    needle: String,
}

fn main() {
    #[cfg(feature = "debug")]
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("simd_needle.log")
            .expect("Unable to open log file");
        tracing_subscriber::fmt()
            .with_writer(file)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    let args = Args::parse();
    println!("haystack: {}\nneedle: {}", args.haystack, args.needle);

    let haystack_path = path::Path::new(&args.haystack);

    let finder = MmapFinder::new(haystack_path, args.needle.into_bytes())
        .expect("Failed to create MmapFinder");

    finder.find_all(SearchAlgo::Simd).for_each(|pos| {
        println!("pos: {}", pos);
    });
}
