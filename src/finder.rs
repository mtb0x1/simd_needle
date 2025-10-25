use std::io::{self, Read};

use crate::search::{bmh_search, naive_search, simd_search, Algorithm};

/// Default buffer size for the finder (8KB)
pub const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// A streaming text finder that searches for a needle in a reader
///
/// Implements an iterator that yields positions of matches in the stream.
/// Supports multiple search algorithms and configurable buffer sizes.
pub struct Finder<R: Read> {
    haystack: R,
    needle: Vec<u8>,
    buffer: Vec<u8>,
    haystack_pos: usize,
    buffer_pos: usize,
    buffer_fill_len: usize,
    algo: Algorithm,
}

impl<R: Read> Finder<R> {
    /// Creates a new Finder with default buffer size
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `algo` - Optional search algorithm to use, defaults to Naive
    pub fn new(haystack: R, needle: Vec<u8>, algo: Option<Algorithm>) -> Self {
        Self::with_buffer_size(haystack, needle, DEFAULT_BUF_SIZE, algo)
    }

    /// Creates a new Finder with custom buffer size
    ///
    /// Buffer size will be adjusted if smaller than needle length
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `buffer_size` - Size of internal buffer for reading
    /// * `algo` - Optional search algorithm to use
    pub fn with_buffer_size(
        haystack: R,
        needle: Vec<u8>,
        mut buffer_size: usize,
        algo: Option<Algorithm>,
    ) -> Self {
        if needle.len() >= buffer_size {
            // Align to cache line,
            //
            buffer_size = needle.len() + 64;
        }
        Self {
            haystack,
            needle,
            buffer: vec![0; buffer_size],
            haystack_pos: 0,
            buffer_pos: 0,
            buffer_fill_len: 0,
            algo: algo.unwrap_or(Algorithm::Naive),
        }
    }

    /// Creates a new Finder with specified algorithm and default buffer size
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `algo` - Search algorithm to use
    pub fn with_algorithm(haystack: R, needle: Vec<u8>, algo: Algorithm) -> Self {
        Self::with_buffer_size(haystack, needle.clone(), DEFAULT_BUF_SIZE, Some(algo))
    }
}

/// Iterator implementation that yields positions of matches in the stream
/// Returns `io::Result<usize>` indicating the position of each match or potential IO errors
impl<R: Read> Iterator for Finder<R> {
    type Item = io::Result<usize>;

    //#[cfg_attr(feature = "hotpath", hotpath::measure)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.needle.is_empty() {
            return None;
        }

        loop {
            if self.buffer_pos >= self.buffer_fill_len {
                // Buffer is exhausted, try to read more data.
                self.haystack_pos += self.buffer_pos;
                self.buffer_fill_len = 0;
                self.buffer_pos = 0;
                match self.haystack.read(&mut self.buffer) {
                    Ok(0) => return None,
                    Ok(n) => self.buffer_fill_len = n,
                    Err(e) => return Some(Err(e)),
                }
            }

            let search_area = &self.buffer[self.buffer_pos..self.buffer_fill_len];
            let found = match self.algo {
                Algorithm::Naive => naive_search(search_area, &self.needle),
                Algorithm::Bmh => bmh_search(search_area, &self.needle),
                Algorithm::Simd => simd_search(search_area, &self.needle),
            };

            if let Some(i) = found {
                let match_pos = self.buffer_pos + i;
                self.buffer_pos = match_pos + 1;
                return Some(Ok(self.haystack_pos + match_pos));
            }

            // No match found, prepare for the next read cycle.
            let tail_len = if self.buffer_fill_len >= self.needle.len() {
                self.needle.len() - 1
            } else {
                self.buffer_fill_len
            };

            let advance = self.buffer_fill_len - tail_len;
            self.haystack_pos += advance;
            self.buffer.copy_within(advance..self.buffer_fill_len, 0);
            self.buffer_fill_len = tail_len;
            self.buffer_pos = 0;

            match self.haystack.read(&mut self.buffer[self.buffer_fill_len..]) {
                Ok(0) => return None,
                Ok(n) => self.buffer_fill_len += n,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
