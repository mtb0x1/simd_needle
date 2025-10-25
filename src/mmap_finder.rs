use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

#[cfg(target_arch = "x86_64")]
use crate::search::simd_search_x86_64;
use crate::search::{bmh_search, kmp_search, naive_search, simd_search, Algorithm};

/// Errors that can occur when working with memory-mapped files
#[derive(Debug)]
pub enum MmapFinderError {
    Io(std::io::Error),
    EmptyNeedle,
}

/// Zero-copy finder for memory-mapped files
///
/// This provides APIs for searching in memory-mapped files without copying data.
/// Memory-mapped files allow efficient access to large files by treating them as if
/// they are in memory, with the OS handling paging transparently.
pub struct MmapFinder {
    mmap: Mmap,
    needle: Vec<u8>,
}

impl MmapFinder {
    /// Create a new MmapFinder from a file path
    ///
    /// # Arguments
    /// * `path` - Path to the file to memory-map
    /// * `needle` - Bytes to search for
    ///
    /// # Returns
    /// Result containing the MmapFinder or an error
    pub fn new<P: AsRef<Path>>(path: P, needle: Vec<u8>) -> Result<Self, MmapFinderError> {
        if needle.is_empty() {
            return Err(MmapFinderError::EmptyNeedle);
        }

        let file = File::open(path).map_err(MmapFinderError::Io)?;
        let mmap = unsafe { Mmap::map(&file).map_err(MmapFinderError::Io)? };

        Ok(Self { mmap, needle })
    }

    /// Create a new MmapFinder from an existing Mmap
    ///
    /// # Arguments
    /// * `mmap` - Memory-mapped region
    /// * `needle` - Bytes to search for
    ///
    /// # Returns
    /// Result containing the MmapFinder or an error
    pub fn from_mmap(mmap: Mmap, needle: Vec<u8>) -> Result<Self, MmapFinderError> {
        if needle.is_empty() {
            return Err(MmapFinderError::EmptyNeedle);
        }

        Ok(Self { mmap, needle })
    }

    /// Find all occurrences of the needle in the memory-mapped file
    ///
    /// # Arguments
    /// * `algo` - Search algorithm to use
    ///
    /// # Returns
    /// Iterator yielding positions of matches
    pub fn find_all(&self, algo: Algorithm) -> MmapFinderIter<'_> {
        MmapFinderIter {
            haystack: &self.mmap,
            needle: &self.needle,
            algo,
            pos: 0,
        }
    }

    /// Find the first occurrence of the needle
    ///
    /// # Arguments
    /// * `algo` - Search algorithm to use
    ///
    /// # Returns
    /// Option containing the position of the first match, or None if not found
    pub fn find_first(&self, algo: Algorithm) -> Option<usize> {
        let search_area = &self.mmap;
        match algo {
            Algorithm::Naive => naive_search(search_area, &self.needle),
            Algorithm::Bmh => bmh_search(search_area, &self.needle),
            Algorithm::Kmp => kmp_search(search_area, &self.needle),
            #[cfg(target_arch = "x86_64")]
            Algorithm::SimdX8664 => simd_search_x86_64(search_area, &self.needle),
            Algorithm::Simd => simd_search(search_area, &self.needle),
        }
    }

    /// Get a reference to the underlying memory-mapped data
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap
    }
}

/// Iterator for finding all occurrences in a memory-mapped file
pub struct MmapFinderIter<'a> {
    haystack: &'a [u8],
    needle: &'a [u8],
    algo: Algorithm,
    pos: usize,
}

impl<'a> Iterator for MmapFinderIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.haystack.len() {
            return None;
        }

        let search_area = &self.haystack[self.pos..];
        let found = match self.algo {
            Algorithm::Naive => naive_search(search_area, self.needle),
            Algorithm::Bmh => bmh_search(search_area, self.needle),
            Algorithm::Kmp => kmp_search(search_area, self.needle),
            #[cfg(target_arch = "x86_64")]
            Algorithm::SimdX8664 => simd_search_x86_64(search_area, self.needle),
            Algorithm::Simd => simd_search(search_area, self.needle),
        };

        match found {
            Some(i) => {
                let match_pos = self.pos + i;
                self.pos = match_pos + 1;
                Some(match_pos)
            }
            None => None,
        }
    }
}

/// Convenience function to search a memory-mapped file
///
/// # Arguments
/// * `path` - Path to the file
/// * `needle` - Bytes to search for
/// * `algo` - Search algorithm to use
///
/// # Returns
/// Vector of positions of all matches
pub fn find_in_file<P: AsRef<Path>>(
    path: P,
    needle: &[u8],
    algo: Algorithm,
) -> Result<Vec<usize>, MmapFinderError> {
    let finder = MmapFinder::new(path, needle.to_vec())?;
    Ok(finder.find_all(algo).collect())
}

/// Convenience function to search an existing Mmap
///
/// # Arguments
/// * `mmap` - Memory-mapped region
/// * `needle` - Bytes to search for
/// * `algo` - Search algorithm to use
///
/// # Returns
/// Iterator yielding positions of all matches
pub fn find_in_mmap<'a>(mmap: &'a Mmap, needle: &'a [u8], algo: Algorithm) -> MmapFinderIter<'a> {
    MmapFinderIter {
        haystack: mmap,
        needle,
        algo,
        pos: 0,
    }
}
