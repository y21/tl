pub fn is_ident(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
        || (b'A'..=b'Z').contains(&c)
        || (b'a'..=b'z').contains(&c)
        || c == b'-'
        || c == b'_'
}

#[inline]
pub fn is_strict_whitespace(c: u8) -> bool {
    c == b' '
}

#[inline]
pub fn find_slow(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&c| c == needle)
}

#[cfg(feature = "simd")]
pub fn find_fast(haystack: &[u8], needle: u8) -> Option<usize> {
    use std::simd::*;

    let len = haystack.len();
    if len < 16 {
        return find_slow(haystack, needle);
    }

    let mut i = 0;
    let needle16 = u8x16::splat(needle);

    while i <= len - 16 {
        let mut bytes = [0; 16];
        for j in 0..16 {
            bytes[j] = unsafe { *haystack.get_unchecked(i + j) };
        }

        let bytes = u8x16::from_array(bytes);
        let eq = bytes.lanes_eq(needle16).to_int();
        let num = unsafe { std::mem::transmute::<_, u128>(eq) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    find_slow(&haystack[i..], needle).map(|x| i + x)
}
