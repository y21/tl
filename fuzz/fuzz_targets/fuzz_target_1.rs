#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = std::str::from_utf8(data) {
        let _ = tl::parse(data, tl::ParserOptions::default());
    }
});
