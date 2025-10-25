use clap::Parser;
use simd_needle::{Finder, SearchAlgo};
use std::path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The haystack string to search in
    haystack: String,

    /// The needle string to search for
    needle: String,
}

fn main() {
    let args = Args::parse();
    eprint!("haystack: {}\nneedle: {}\n", args.haystack, args.needle);

    let haystack_path = path::Path::new(&args.haystack);
    let needle = args.needle.into_bytes();
    let haystack_reader = std::fs::read(&haystack_path).unwrap();

    let mut finder = Finder::with_algorithm(&haystack_reader[..], needle, SearchAlgo::Simd);

    while let Some(Ok(pos)) = finder.next() {
        println!("pos: {}", pos);
    }
}
