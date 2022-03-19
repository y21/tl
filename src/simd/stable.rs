use crate::simd::fallback;

#[inline]
pub fn is_closing(needle: u8) -> bool {
    let eq1 = needle == b'/';
    let eq2 = needle == b'>';

    eq1 | eq2
}

pub fn find4(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
    #[inline(never)]
    #[cold]
    fn unlikely_search(haystack: &[u8], needle: [u8; 4]) -> Option<usize> {
        fallback::find_multi(haystack, needle)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    if len < 16 {
        return unlikely_search(haystack, needle);
    }

    let mut i = 0usize;
    while i <= len - 16 {
        let mut mask = 0u16;

        for j in 0..16 {
            let c = unsafe { *ptr.add(i + j) };
            mask |= ((c == needle[0]) as u16) << j;
            mask |= ((c == needle[1]) as u16) << j;
            mask |= ((c == needle[2]) as u16) << j;
            mask |= ((c == needle[3]) as u16) << j;
        }

        if mask != 0 {
            let index = mask.trailing_zeros() as usize;
            return Some(i + index);
        }

        i += 16;
    }

    fallback::find_multi(&haystack[i..], needle).map(|x| x + i)
}

pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    #[inline(never)]
    #[cold]
    fn unlikely_search(haystack: &[u8], needle: u8) -> Option<usize> {
        fallback::find(haystack, needle)
    }

    let len = haystack.len();
    let ptr = haystack.as_ptr();
    if len < 16 {
        return unlikely_search(haystack, needle);
    }

    let mut i = 0usize;
    while i <= len - 16 {
        let mut mask = 0u16;

        for j in 0..16 {
            let c = unsafe { *ptr.add(i + j) };
            mask |= ((c == needle) as u16) << j;
        }

        if mask != 0 {
            let index = mask.trailing_zeros() as usize;
            return Some(i + index);
        }

        i += 16;
    }

    fallback::find(&haystack[i..], needle).map(|x| x + i)
}
