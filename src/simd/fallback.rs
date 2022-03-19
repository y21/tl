use crate::util;

pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&c| c == needle)
}

pub fn find_multi<const N: usize>(haystack: &[u8], needle: [u8; N]) -> Option<usize> {
    haystack.iter().position(|c| needle.contains(c))
}

pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&c| !util::is_ident(c))
}
