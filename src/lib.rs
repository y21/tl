mod bytes;
mod parser;
mod stream;
#[cfg(test)]
mod tests;
mod util;
mod vdom;

use parser::Parser;
use vdom::VDom;

pub fn parse(input: &str) -> VDom<'_> {
    VDom::from(Parser::new(input).parse())
}
