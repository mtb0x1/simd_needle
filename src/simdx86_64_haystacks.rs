use clap::Parser;
use simd_needle::{Finder, FinderTrait, SearchAlgo};
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

//FIXME : better way than this when we are on not x86_64 architecture
#[cfg(target_arch = "x86_64")]
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
    let needle = args.needle.into_bytes();
    let haystack_reader = BufReader::new(File::open(haystack_path).unwrap());

    let finder = Finder::with_algorithm(haystack_reader, needle, SearchAlgo::SimdX8664).unwrap();

    for pos in finder.flatten() {
        println!("pos: {}", pos);
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("This binary is only supported on x86_64 architecture.");
}
