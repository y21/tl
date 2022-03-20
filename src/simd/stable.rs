use crate::simd::fallback;

/// Optimized function for checking if a byte is a closing tag
#[inline]
pub fn is_closing(needle: u8) -> bool {
    let eq1 = needle == b'/';
    let eq2 = needle == b'>';

    eq1 | eq2
}

/// Optimized, stable function for finding any byte in `haystack`
pub fn find_multi<const N: usize>(haystack: &[u8], needle: [u8; N]) -> Option<usize> {
    let mut index = 0;

    for (i, chunk) in haystack.chunks_exact(16).enumerate() {
        index = i * 16;
        let mut mask = 0u16;

        for (j, &byte) in chunk.into_iter().enumerate() {
            for k in 0..N {
                mask |= ((byte == needle[k]) as u16) << j;
            }
        }

        if mask != 0 {
            let local_index = mask.trailing_zeros() as usize;
            return Some(index + local_index);
        }
    }

    fallback::find_multi(&haystack[index..], needle).map(|x| x + index)
}

/// Optimized, stable function for finding a byte in `haystack`
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    let mut index = 0;

    for (i, chunk) in haystack.chunks_exact(16).enumerate() {
        index = i * 16;
        let mut mask = 0u16;

        for (j, &byte) in chunk.into_iter().enumerate() {
            mask |= ((byte == needle) as u16) << j;
        }

        if mask != 0 {
            let local_index = mask.trailing_zeros() as usize;
            return Some(index + local_index);
        }
    }

    fallback::find(&haystack[index..], needle).map(|x| x + index)
}
