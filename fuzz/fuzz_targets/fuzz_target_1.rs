#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tl;

fuzz_target!(|data: &str| {
    let _ = tl::parse(data, tl::ParserOptions::default());
});
