#![cfg_attr(feature = "simd", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod bytes;
/// Errors that occur throughout the crate
pub mod errors;
/// Inline data structures
pub mod inline;
mod parser;
/// Query selector API
pub mod queryselector;
mod stream;
#[cfg(test)]
mod tests;
mod util;
mod vdom;

#[doc(hidden)]
#[cfg(feature = "__INTERNALS_DO_NOT_USE")]
pub mod simd;
#[cfg(not(feature = "__INTERNALS_DO_NOT_USE"))]
mod simd;

pub use bytes::Bytes;
pub use errors::ParseError;
pub use parser::*;
use queryselector::Selector;
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
    let mut parser = Parser::new(input, options);
    parser.parse()?;
    Ok(VDom::from(parser))
}

/// Parses a query selector
///
/// # Example
/// ```
/// # use tl::queryselector::selector::Selector;
/// let selector = tl::parse_query_selector("div#test");
///
/// match selector {
///     Some(Selector::And(left, right)) => {
///         assert!(matches!(&*left, Selector::Tag(b"div")));
///         assert!(matches!(&*right, Selector::Id(b"test")));
///     },
///     _ => unreachable!()
/// }
/// ```
pub fn parse_query_selector(input: &str) -> Option<Selector<'_>> {
    let selector = queryselector::Parser::new(input.as_bytes()).parse_selectors_group()?;
    Some(selector)
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
pub unsafe fn parse_owned(input: String, options: ParserOptions) -> Result<VDomGuard, ParseError> {
    VDomGuard::parse(input, options)
}
