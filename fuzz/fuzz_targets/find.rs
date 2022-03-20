#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

fuzz_target!(|data: (&[u8], u8)| {
    let (haystack, needle) = data;
    tl::simd::find(haystack, needle);
});
