//! tl is an efficient and easy to use HTML parser written in Rust.
//!
//! It does minimal to no copying during parsing by borrowing parts of the input string.
//! Additionally, it keeps track of parsed elements and inserts elements with an `id` attribute
//! in an internal HashMap, which makes `get_element_by_id` as well as `get_elements_by_class_name` very fast (`O(1)`).
//!
//! ## Examples
//! Finding an element by its `id` attribute and printing the inner text:
//! ```rust
//! fn main() {
//!     let input = r#"<p id="text">Hello</p>"#;
//!     let dom = tl::parse(input);
//!
//!     let element = dom.get_element_by_id("text").expect("Failed to find element");
//!
//!     println!("Inner text: {}", element.inner_text());
//! }
//! ```
//!
//! ## Bytes
//! Some methods return a `Bytes` struct, which is an internal struct that is used to borrow
//! a part of the input string. This is mainly used over a raw `&[u8]` for its `Debug` implementation.

#![deny(missing_docs)]

mod bytes;
mod parser;
mod stream;
#[cfg(test)]
mod tests;
mod util;
mod vdom;

pub use bytes::{AsBytes, Bytes};
use parser::Parser;
pub use parser::{Attributes, HTMLTag, HTMLVersion, Node, Tree};
pub use vdom::VDom;

/// Parses the given input string
/// This is the "entry point" and function you will call to parse HTML
pub fn parse(input: &str) -> VDom<'_> {
    VDom::from(Parser::new(input).parse())
}
