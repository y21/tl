mod bytes;
mod parser;
mod stream;
mod util;
mod vdom;
#[cfg(test)]
mod tests;

use parser::Parser;
use vdom::VDom;

pub fn parse(input: &str) -> VDom<'_> {
    VDom::from(Parser::new(input).parse())
}
