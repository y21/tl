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
    pub fn get_element_by_id<'b, S: ?Sized>(&'b self, id: &'b S) -> Option<&'b Rc<Node<'a>>>
    where
        S: AsBytes,
    {
        self.parser.ids.get(&id.as_bytes())
    }

    pub fn get_elements_by_class_name<'b, S: ?Sized>(&'b self, id: &'b S) -> Option<&'b Vec<Rc<Node<'a>>>>
    where
        S: AsBytes
    {
        self.parser.classes.get(&id.as_bytes())
    }

    pub fn children(&self) -> &Tree<'a> {
        &self.parser.ast
    }

    pub fn version(&self) -> Option<HTMLVersion> {
        self.parser.version
    }
}
