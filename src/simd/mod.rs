use crate::util;

/// Fallback functions, used for the last chunk not divisible by the chunk sice
pub mod fallback;
/// nightly-only functions using portable_simd
#[cfg(feature = "simd")]
pub mod nightly;
/// Stable, "fallback" functions that this library uses as a fallback until portable_simd becomes stable
#[cfg(not(feature = "simd"))]
pub mod stable;

macro_rules! decide {
    ($nightly:expr, $stable:expr) => {{
        #[cfg(feature = "simd")]
        {
            $nightly
        }
        #[cfg(not(feature = "simd"))]
        {
            $stable
        }
    }};
}

/// Checks if the given byte is a "closing" byte (/ or >)
#[inline]
pub fn is_closing(needle: u8) -> bool {
    decide!(nightly::is_closing(needle), stable::is_closing(needle))
}

/// Searches for the first non-identifier in `haystack`
#[inline]
pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    decide!(
        nightly::search_non_ident(haystack),
        fallback::search_non_ident(haystack)
    )
}

/// Searches for the first occurence in `haystack`
#[inline]
pub fn find4(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
    decide!(
        nightly::find4(haystack, needle),
        stable::find_multi(haystack, needle)
    )
}

/// Searches for the first occurence of `needle` in `haystack`
#[inline]
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    decide!(
        nightly::find(haystack, needle),
        stable::find(haystack, needle)
    )
}

/// Checks if the ASCII characters in `haystack` match `needle` (case insensitive)
pub fn matches_case_insensitive<const N: usize>(haystack: &[u8], needle: [u8; N]) -> bool {
    if haystack.len() != N {
        return false;
    }

    // LLVM seems to already generate pretty good SIMD even without explicit use

    let mut mask = true;
    for i in 0..N {
        mask &= util::to_lower(haystack[i]) == needle[i];
    }
    mask
}
