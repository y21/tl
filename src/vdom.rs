use crate::{bytes::AsBytes, parser::HTMLVersion};
use crate::parser::{Node, Parser, Tree};
use std::rc::Rc;

#[derive(Debug)]
pub struct VDom<'a> {
    parser: Parser<'a>
}

impl<'a> From<Parser<'a>> for VDom<'a> {
    fn from(parser: Parser<'a>) -> Self {
        Self { parser }
    }
}

impl<'a> VDom<'a> {
    pub fn get_element_by_id<S: ?Sized>(&self, id: &S) -> Option<Rc<Node<'a>>>
    where
        S: AsBytes,
    {
        let id = id.as_bytes();

        self.parser.ids.get(&id).cloned()
    }

    pub fn children(&self) -> &Tree<'a> {
        &self.parser.ast
    }

    pub fn version(&self) -> Option<HTMLVersion> {
        self.parser.version
    }
}
