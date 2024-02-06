use std::{ptr, simd::*, simd::cmp::{SimdPartialEq,  SimdPartialOrd}};

use crate::simd::fallback;

/// Optimized function for finding a byte in `haystack`
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    #[inline(never)]
    #[cold]
    fn unlikely_find(haystack: &[u8], needle: u8) -> Option<usize> {
        fallback::find(haystack, needle)
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
        let eq = bytes.simd_eq(needle16).to_int();
        let num = unsafe { std::mem::transmute::<Simd<i8, 16>, u128>(eq) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    fallback::find(&haystack[i..], needle).map(|x| i + x)
}

/// Optimized function for finding one of 4 bytes in `haystack`
pub fn find4(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
    #[inline(never)]
    #[cold]
    fn unlikely_find(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
        fallback::find_multi(haystack, needle)
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

        let eq1 = bytes.simd_eq(needle16a);
        let eq2 = bytes.simd_eq(needle16b);
        let eq3 = bytes.simd_eq(needle16c);
        let eq4 = bytes.simd_eq(needle16d);
        let or = (eq1 | eq2 | eq3 | eq4).to_int();
        let num = unsafe { std::mem::transmute::<i8x16, u128>(or) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    fallback::find_multi(&haystack[i..], needle).map(|x| i + x)
}

/// Optimized function for searching for the first non-identifier
pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    #[inline(never)]
    #[cold]
    fn unlikely_search(haystack: &[u8]) -> Option<usize> {
        fallback::search_non_ident(haystack)
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

        let ge_zero = bytes.simd_ge(needle_zero);
        let le_nine = bytes.simd_le(needle_nine);
        let digit = ge_zero & le_nine;

        let ge_lc_a = bytes.simd_ge(needle_lc_a);
        let le_lc_z = bytes.simd_le(needle_lc_z);
        let lowercase = ge_lc_a & le_lc_z;

        let ge_uc_a = bytes.simd_ge(needle_uc_a);
        let le_uc_z = bytes.simd_le(needle_uc_z);
        let uppercase = ge_uc_a & le_uc_z;

        let eq_minus = bytes.simd_eq(needle_minus);
        let eq_underscore = bytes.simd_eq(needle_underscore);
        let symbol = eq_minus | eq_underscore;

        let or = !(digit | lowercase | uppercase | symbol).to_int();

        let num = unsafe { std::mem::transmute::<i8x16, u128>(or) };
        if num != 0 {
            return Some(i + (num.trailing_zeros() >> 3) as usize);
        }

        i += 16;
    }

    fallback::search_non_ident(&haystack[i..]).map(|x| i + x)
}

/// Optimized function for checking if a byte is a closing tag
#[inline]
pub fn is_closing(needle: u8) -> bool {
    (needle == b'/') | (needle == b'>')
}
