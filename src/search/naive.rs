/// A simple, naive byte-slice search that returns the index of the first
/// occurrence of `needle` inside `haystack`, or `None` if not found.
///
/// This mirrors the previous behaviour using `.windows(...).position(...)`
/// but is extracted so the algorithm can be named and replaced later.
pub fn naive_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    let last_start = haystack.len() - needle.len();
    (0..=last_start).find(|&i| &haystack[i..i + needle.len()] == needle)
}
