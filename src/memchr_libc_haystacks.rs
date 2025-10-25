use clap::Parser;
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
    println!("haystack: {}\nneedle: {}", args.haystack, args.needle);

    let haystack_path = path::Path::new(&args.haystack);
    let needle = args.needle.into_bytes();

    //file reader on haystack
    let haystack_reader = std::fs::read(haystack_path).unwrap();

    let n = needle.len();
    if n == 0 || haystack_reader.len() < n {
        return;
    }

    let mut start = haystack_reader.as_ptr();
    let end = unsafe { start.add(haystack_reader.len()) };

    while start < end {
        let remaining = unsafe { end.offset_from(start) as usize };

        // Search for the first byte of the needle using libc::memchr
        let res =
            unsafe { libc::memchr(start as *const libc::c_void, needle[0] as i32, remaining) };
        if res.is_null() {
            break;
        }

        let pos = unsafe { (res as *const u8).offset_from(haystack_reader.as_ptr()) as usize };

        // Check if the rest of the bytes match the needle
        if pos + n <= haystack_reader.len() && haystack_reader[pos..pos + n] == needle {
            println!("pos: {}", pos);
        }

        // Move one byte forward (allows overlapping matches)
        start = unsafe { (res as *const u8).add(1) };
    }
}
