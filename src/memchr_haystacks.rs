use std::path;

use clap::Parser;
use memchr::memmem;

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

    //file reader on haystack
    let haystack_reader = std::fs::read(&haystack_path).unwrap();

    let mut finder = memmem::find_iter(&haystack_reader, &needle);

    while let Some(pos) = finder.next() {
        eprint!("pos: {}\n", pos);
    }
}
