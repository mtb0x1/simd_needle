use std::io::{self, Read};

#[cfg(target_arch = "x86_64")]
use crate::search::simd_search_x86_64;
use crate::search::{bmh_search, kmp_search, naive_search, simd_search, Algorithm};

#[derive(Debug)]
pub enum FinderError {
    EmptyNeedle,
    BufferTooSmall,
}

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
    requested_buffer_size: usize,
}

pub trait FinderTrait<R: Read> {
    fn new(haystack: R, needle: Vec<u8>, algo: Option<Algorithm>) -> Result<Self, FinderError>
    where
        Self: Sized;
    fn with_buffer_size(
        haystack: R,
        needle: Vec<u8>,
        buffer_size: usize,
        algo: Option<Algorithm>,
    ) -> Result<Self, FinderError>
    where
        Self: Sized;
    fn with_algorithm(haystack: R, needle: Vec<u8>, algo: Algorithm) -> Result<Self, FinderError>
    where
        Self: Sized;
}

impl<R: Read> FinderTrait<R> for Finder<R> {
    /// Creates a new Finder with default buffer size
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `algo` - Optional search algorithm to use, defaults to Naive
    fn new(haystack: R, needle: Vec<u8>, algo: Option<Algorithm>) -> Result<Self, FinderError> {
        if needle.is_empty() {
            return Err(FinderError::EmptyNeedle);
        }
        if needle.len() > DEFAULT_BUF_SIZE {
            return Err(FinderError::BufferTooSmall);
        }
        Self::with_buffer_size(haystack, needle, DEFAULT_BUF_SIZE, algo)
    }

    /// Creates a new Finder with custom buffer size
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `buffer_size` - Size of internal buffer for reading
    /// * `algo` - Optional search algorithm to use
    fn with_buffer_size(
        haystack: R,
        needle: Vec<u8>,
        mut buffer_size: usize,
        algo: Option<Algorithm>,
    ) -> Result<Self, FinderError> {
        if needle.is_empty() {
            return Err(FinderError::EmptyNeedle);
        }
        if needle.len() > buffer_size {
            return Err(FinderError::BufferTooSmall);
        }
        let requested_buffer_size = buffer_size;
        buffer_size += needle.len() - 1;
        Ok(Self {
            haystack,
            needle,
            buffer: vec![0; buffer_size],
            haystack_pos: 0,
            buffer_pos: 0,
            buffer_fill_len: 0,
            algo: algo.unwrap_or(Algorithm::Naive),
            requested_buffer_size,
        })
    }

    /// Creates a new Finder with specified algorithm and default buffer size
    ///
    /// # Arguments
    /// * `haystack` - The source to read from and search in
    /// * `needle` - Bytes to search for
    /// * `algo` - Search algorithm to use
    fn with_algorithm(haystack: R, needle: Vec<u8>, algo: Algorithm) -> Result<Self, FinderError> {
        if needle.is_empty() {
            return Err(FinderError::EmptyNeedle);
        }
        if needle.len() > DEFAULT_BUF_SIZE {
            return Err(FinderError::BufferTooSmall);
        }
        Self::with_buffer_size(haystack, needle, DEFAULT_BUF_SIZE, Some(algo))
    }
}

/// Iterator implementation that yields positions of matches in the stream
/// Returns `io::Result<usize>` indicating the position of each match or potential IO errors
impl<R: Read> Iterator for Finder<R> {
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.buffer_pos >= self.buffer_fill_len {
                // Buffer is exhausted, try to read more data.
                self.haystack_pos += self.buffer_pos;
                self.buffer_fill_len = 0;
                self.buffer_pos = 0;
                match self.haystack.read(&mut self.buffer) {
                    Ok(0) => return None,
                    Ok(n) => {
                        self.buffer_fill_len = n;
                        // If needle is longer than what we could read, no match is possible
                        if self.haystack_pos == 0 && n < self.needle.len() {
                            return None;
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }

            let search_area = &self.buffer[self.buffer_pos..self.buffer_fill_len];
            let found = match self.algo {
                Algorithm::Naive => naive_search(search_area, &self.needle),
                Algorithm::Bmh => bmh_search(search_area, &self.needle),
                Algorithm::Kmp => kmp_search(search_area, &self.needle),
                #[cfg(target_arch = "x86_64")]
                Algorithm::SimdX8664 => simd_search_x86_64(search_area, &self.needle),
                Algorithm::Simd => simd_search(search_area, &self.needle),
            };

            if let Some(i) = found {
                let match_pos = self.buffer_pos + i;
                self.buffer_pos = match_pos + 1;
                return Some(Ok(self.haystack_pos + match_pos));
            }

            // No match found
            if self.buffer_pos + self.needle.len() > self.buffer_fill_len {
                // Need more data
                if self.buffer_fill_len >= self.buffer.len() {
                    // Buffer full, copy tail to start
                    let tail_len = self.needle.len() - 1;
                    self.buffer
                        .copy_within(self.buffer_fill_len - tail_len..self.buffer_fill_len, 0);
                    self.buffer_fill_len = tail_len;
                    self.buffer_pos = 0;
                    self.haystack_pos += self.buffer.len() - tail_len;
                }
                match self.haystack.read(&mut self.buffer[self.buffer_fill_len..]) {
                    Ok(0) => return None,
                    Ok(n) => self.buffer_fill_len += n,
                    Err(e) => return Some(Err(e)),
                }
            } else {
                // Advance the window
                let tail_len = self.needle.len() - 1;
                let advance = (self.buffer_fill_len - self.buffer_pos) - tail_len;
                self.haystack_pos += advance;
                self.buffer_pos += advance;
            }
        }
    }
}
