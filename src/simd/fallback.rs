use crate::util;

/// Fallback for finding a byte
#[inline(never)]
#[cold]
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&c| c == needle)
}

/// Fallback for finding any byte
#[inline(never)]
#[cold]
pub fn find_multi<const N: usize>(haystack: &[u8], needle: [u8; N]) -> Option<usize> {
    haystack.iter().position(|c| needle.contains(c))
}

/// Fallback for searching for the first non-identifier
#[inline(never)]
#[cold]
pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&c| !util::is_ident(c))
}
