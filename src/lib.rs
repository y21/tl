#![cfg_attr(feature = "simd", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod bytes;
/// Errors that occur throughout the crate
pub mod errors;
/// Inline data structures
pub mod inline;
mod parser;
mod queryselector;
mod stream;
#[cfg(test)]
mod tests;
mod util;
mod vdom;

pub use bytes::Bytes;
pub use errors::ParseError;
pub use parser::*;
pub use vdom::{VDom, VDomGuard};

/// Parses the given input string
///
/// This is the "entry point" and function that is called to parse HTML.
/// The input string must be kept alive, and must outlive `VDom`.
/// If you need an "owned" version that takes an input string and can be kept around forever,
/// consider using `parse_owned()`.
///
/// # Errors
/// Throughout the parser it is assumed that spans never overflow a `u32`.
/// To prevent this, this function will return an error if the input string length would overflow a `u32`.
/// If the input string length fits in a `u32`, then it is safe to assume that none of the substrings can overflow a `u32`.
///
/// # Example
/// ```
/// # use tl::*;
/// let dom = parse("<div>Hello, world!</div>", ParserOptions::default()).unwrap();
/// assert_eq!(dom.query_selector("div").unwrap().count(), 1);
/// ```
pub fn parse(input: &str, options: ParserOptions) -> Result<VDom<'_>, ParseError> {
    Ok(VDom::from(Parser::new(input, options).parse()?))
}

/// Parses the given input string and returns an owned, RAII guarded DOM
///
/// # Errors
/// See [parse]
///
/// # Safety
/// This uses `unsafe` code to create a self-referential-like struct.
/// The given input string is first leaked and turned into raw pointer, and its lifetime will be promoted to 'static.
/// Once `VDomGuard` goes out of scope, the string will be freed.
/// It should not be possible to cause UB in its current form and might become a safe function in the future.
pub unsafe fn parse_owned<'a>(
    input: String,
    options: ParserOptions,
) -> Result<VDomGuard<'a>, ParseError> {
    VDomGuard::parse(input, options)
}
