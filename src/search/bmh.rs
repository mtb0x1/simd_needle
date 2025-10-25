/// Boyer-Moore-Horspool search (simple implementation).
///
/// # Arguments
/// * `haystack` - The data to search in
/// * `needle` - The pattern to search for
///
/// # Returns
/// * `Some(usize)` - Index of the first match
/// * `None` - If no match is found or needle is empty
pub fn bmh_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let m = needle.len();
    if m == 0 || haystack.len() < m {
        return None;
    }

    // Build bad-character shift table
    let mut shift = [m; 256usize];
    for i in 0..m - 1 {
        shift[needle[i] as usize] = m - 1 - i;
    }

    let mut i = 0usize;
    while i + m <= haystack.len() {
        let mut j = (m - 1) as isize;
        while j >= 0 && haystack[i + j as usize] == needle[j as usize] {
            j -= 1;
        }
        if j < 0 {
            return Some(i);
        }
        let next_byte = haystack[i + m - 1];
        i += shift[next_byte as usize];
    }
    None
}
