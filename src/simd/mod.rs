use crate::util;

pub mod fallback;
#[cfg(feature = "simd")]
pub mod nightly;
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

#[inline]
pub fn is_closing(needle: u8) -> bool {
    decide!(nightly::is_closing(needle), stable::is_closing(needle))
}

#[inline]
pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    decide!(
        nightly::search_non_ident(haystack),
        fallback::search_non_ident(haystack)
    )
}

#[inline]
pub fn find4(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
    decide!(
        nightly::find4(haystack, needle),
        stable::find4(haystack, needle)
    )
}

#[inline]
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    decide!(
        nightly::find(haystack, needle),
        stable::find(haystack, needle)
    )
}

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
