//! tl is an efficient and easy to use HTML parser written in Rust.
//!
//! It does minimal to no copying during parsing by borrowing parts of the input string.
//! Additionally, it keeps track of parsed elements and stores elements with an id attribute
//! in an internal HashMap, which makes element lookups by ID/class name very fast.
//!
//! ## Examples
//! Finding an element by its id attribute and printing the inner text:
//! ```rust
//! let input = r#"<p id="text">Hello</p>"#;
//! let dom = tl::parse(input);
//!
//! let element = dom.get_element_by_id("text").expect("Failed to find element");
//!
//! println!("Inner text: {}", element.inner_text());
//! ```
//!
//! ## Owned DOM
//! Calling `tl::parse()` returns a DOM struct that borrows from the input string, which means that the string must be kept alive.
//! The input string must outlive this DOM. If this is not acceptable or you need to keep the DOM around for longer,
//! consider using `tl::parse_owned()`.
//! `VDomGuard` takes ownership over the string, which means you don't have to keep the string around.
//! ```rust
//! // Notice how it takes ownership over the string:
//! let dom_guard = tl::parse_owned(String::from(r#"<p id="text">Hello</p>"#));
//! 
//! // Obtain reference to underlying VDom
//! let dom = dom_guard.get_ref();
//! 
//! // Now, use `dom` as you would if it was a regular `VDom`
//! let element = dom.get_element_by_id("text").expect("Failed to find element");
//! 
//! println!("Inner text: {}", element.inner_text());
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
pub use vdom::{VDom, VDomGuard};

/// Parses the given input string
/// 
/// This is the "entry point" and function you will call to parse HTML.
/// The input string must be kept alive, and must outlive `VDom`.
/// If you need an "owned" version that takes an input string and can be kept around forever,
/// consider using `parse_owned()`.
pub fn parse(input: &str) -> VDom<'_> {
    VDom::from(Parser::new(input).parse())
}

/// Parses the given input string and returns an owned, RAII guarded DOM
///
/// ## Safety
/// This uses a lot of `unsafe` behind the scenes to create a self-referential-like struct.
/// The given input string is first leaked and turned into raw pointer, and its lifetime will be promoted to 'static.
/// Once `VDomGuard` goes out of scope, the string will be freed.
/// It should not be possible to cause UB in its current form and might become a safe function in the future.
pub unsafe fn parse_owned<'a>(input: String) -> VDomGuard<'a> {
    VDomGuard::parse(input)
}