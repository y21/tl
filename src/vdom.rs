use crate::parser::{Node, Parser, Tree};
use crate::{bytes::AsBytes, parser::HTMLVersion};
use std::rc::Rc;

/// VDom represents a [Document Object Model](https://developer.mozilla.org/en/docs/Web/API/Document_Object_Model)
#[derive(Debug)]
pub struct VDom<'a> {
    /// Internal parser
    parser: Parser<'a>,
}

impl<'a> From<Parser<'a>> for VDom<'a> {
    fn from(parser: Parser<'a>) -> Self {
        Self { parser }
    }
}

impl<'a> VDom<'a> {
    /// Finds an element by its `id` attribute. This operation is O(1), as it's only a HashMap lookup
    pub fn get_element_by_id<'b, S: ?Sized>(&'b self, id: &'b S) -> Option<&'b Rc<Node<'a>>>
    where
        S: AsBytes,
    {
        self.parser.ids.get(&id.as_bytes())
    }

    /// Returns a list of elements that match a given class name. This operation is O(1), as it's only a HashMap lookup
    pub fn get_elements_by_class_name<'b, S: ?Sized>(
        &'b self,
        id: &'b S,
    ) -> Option<&'b Vec<Rc<Node<'a>>>>
    where
        S: AsBytes,
    {
        self.parser.classes.get(&id.as_bytes())
    }

    /// Returns all subnodes ("children") of this DOM
    pub fn children(&self) -> &Tree<'a> {
        &self.parser.ast
    }

    /// Returns the HTML version.
    /// This is determined by the `<!DOCTYPE>` tag
    pub fn version(&self) -> Option<HTMLVersion> {
        self.parser.version
    }
}
