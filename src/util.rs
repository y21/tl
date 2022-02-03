#[inline(never)]
pub fn is_ident(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
        || (b'A'..=b'Z').contains(&c)
        || (b'a'..=b'z').contains(&c)
        || c == b'-'
        || c == b'_'
}

#[inline(never)]
pub fn find_slow(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&c| c == needle)
}

#[inline(never)]
pub fn find_multi_slow<const N: usize>(haystack: &[u8], needle: [u8; N]) -> Option<usize> {
    haystack.iter().position(|c| needle.contains(c))
}

#[cfg(feature = "simd")]
pub fn find_fast(haystack: &[u8], needle: u8) -> Option<usize> {
    use std::{ptr, simd::*};

    #[inline(never)]
    #[cold]
    fn unlikely_find(haystack: &[u8], needle: u8) -> Option<usize> {
        find_slow(haystack, needle)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    if len < 16 {
        return unlikely_find(haystack, needle);
    }

    let mut i = 0;
    let needle16 = u8x16::splat(needle);

    while i <= len - 16 {
        let mut bytes = [0; 16];
        unsafe { ptr::copy_nonoverlapping(ptr.add(i), bytes.as_mut_ptr(), 16) };

        let bytes = u8x16::from_array(bytes);
        let eq = bytes.lanes_eq(needle16).to_int();
        let num = unsafe { std::mem::transmute::<Simd<i8, 16>, u128>(eq) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    find_slow(&haystack[i..], needle).map(|x| i + x)
}

#[cfg(feature = "simd")]
pub fn find_fast_4(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
    use std::{ptr, simd::*};

    #[inline(never)]
    #[cold]
    fn unlikely_find(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
        find_multi_slow(haystack, needle)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    if len < 16 {
        return unlikely_find(haystack, needle);
    }

    let mut i = 0;
    let needle16a = u8x16::splat(needle[0]);
    let needle16b = u8x16::splat(needle[1]);
    let needle16c = u8x16::splat(needle[2]);
    let needle16d = u8x16::splat(needle[3]);

    while i <= len - 16 {
        let mut bytes = [0; 16];
        unsafe { ptr::copy_nonoverlapping(ptr.add(i), bytes.as_mut_ptr(), 16) };

        let bytes = u8x16::from_array(bytes);

        let eq1 = bytes.lanes_eq(needle16a);
        let eq2 = bytes.lanes_eq(needle16b);
        let eq3 = bytes.lanes_eq(needle16c);
        let eq4 = bytes.lanes_eq(needle16d);
        let or = (eq1 | eq2 | eq3 | eq4).to_int();
        let num = unsafe { std::mem::transmute::<i8x16, u128>(or) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    find_multi_slow(&haystack[i..], needle).map(|x| i + x)
}

pub fn search_non_ident_slow(haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&c| !is_ident(c))
}

#[cfg(feature = "simd")]
pub fn search_non_ident_fast(haystack: &[u8]) -> Option<usize> {
    use std::{ptr, simd::*};

    #[inline(never)]
    #[cold]
    fn unlikely_search(haystack: &[u8]) -> Option<usize> {
        search_non_ident_slow(haystack)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    if len < 16 {
        return unlikely_search(haystack);
    }

    let mut i = 0;
    let needle_zero = u8x16::splat(b'0');
    let needle_nine = u8x16::splat(b'9');
    let needle_lc_a = u8x16::splat(b'a');
    let needle_lc_z = u8x16::splat(b'z');
    let needle_uc_a = u8x16::splat(b'A');
    let needle_uc_z = u8x16::splat(b'Z');
    let needle_minus = u8x16::splat(b'-');
    let needle_underscore = u8x16::splat(b'_');

    while i <= len - 16 {
        let mut bytes = [0; 16];
        unsafe { ptr::copy_nonoverlapping(ptr.add(i), bytes.as_mut_ptr(), 16) };

        let bytes = u8x16::from_array(bytes);

        let ge_zero = bytes.lanes_ge(needle_zero);
        let le_nine = bytes.lanes_le(needle_nine);
        let digit = ge_zero & le_nine;

        let ge_lc_a = bytes.lanes_ge(needle_lc_a);
        let le_lc_z = bytes.lanes_le(needle_lc_z);
        let lowercase = ge_lc_a & le_lc_z;

        let ge_uc_a = bytes.lanes_ge(needle_uc_a);
        let le_uc_z = bytes.lanes_le(needle_uc_z);
        let uppercase = ge_uc_a & le_uc_z;

        let eq_minus = bytes.lanes_eq(needle_minus);
        let eq_underscore = bytes.lanes_eq(needle_underscore);
        let symbol = eq_minus | eq_underscore;

        let or = !(digit | lowercase | uppercase | symbol).to_int();

        let num = unsafe { std::mem::transmute::<i8x16, u128>(or) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    search_non_ident_slow(&haystack[i..]).map(|x| i + x)
}

#[inline]
pub fn is_closing(needle: u8) -> bool {
    #[cfg(feature = "simd")]
    {
        use std::simd::u8x4;
        let sc = u8x4::from_array([b'/', b'>', 0, 0]);
        let needle = u8x4::splat(needle);
        let eq = needle.lanes_eq(sc);
        eq.any()
    }
    #[cfg(not(feature = "simd"))]
    {
        let eq1 = needle == b'/';
        let eq2 = needle == b'>';

        eq1 | eq2
    }
}

#[inline(always)]
fn to_lower(byte: u8) -> u8 {
    let is_upper = (byte >= b'A' && byte <= b'Z') as u8;
    let lower = is_upper * 0x20;
    byte + lower
}

pub fn matches_case_insensitive<const N: usize>(haystack: &[u8], needle: [u8; N]) -> bool {
    if haystack.len() != N {
        return false;
    }

    // LLVM seems to already generate pretty good SIMD even without explicit use

    let mut mask = true;
    for i in 0..N {
        mask &= to_lower(haystack[i]) == needle[i];
    }
    mask
}
