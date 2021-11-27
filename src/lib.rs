#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod bytes;
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
pub use parser::*;
pub use vdom::{VDom, VDomGuard};

/// Parses the given input string
///
/// This is the "entry point" and function you will call to parse HTML.
/// The input string must be kept alive, and must outlive `VDom`.
/// If you need an "owned" version that takes an input string and can be kept around forever,
/// consider using `parse_owned()`.
pub fn parse(input: &str, options: ParserOptions) -> VDom<'_> {
    VDom::from(Parser::new(input, options).parse())
}

/// Parses the given input string and returns an owned, RAII guarded DOM
///
/// ## Safety
/// This uses a lot of `unsafe` behind the scenes to create a self-referential-like struct.
/// The given input string is first leaked and turned into raw pointer, and its lifetime will be promoted to 'static.
/// Once `VDomGuard` goes out of scope, the string will be freed.
/// It should not be possible to cause UB in its current form and might become a safe function in the future.
pub unsafe fn parse_owned<'a>(input: String, options: ParserOptions) -> VDomGuard<'a> {
    VDomGuard::parse(input, options)
}
