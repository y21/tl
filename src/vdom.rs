use crate::bytes::{AsBytes, BorrowedBytes};
use crate::parser::{Node, Parser, Tree};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct VDom<'a> {
    ast: Tree<'a>,
    ids: HashMap<BorrowedBytes<'a>, Rc<Node<'a>>>,
    classes: HashMap<BorrowedBytes<'a>, Rc<Node<'a>>>,
}

impl<'a> From<Parser<'a>> for VDom<'a> {
    fn from(p: Parser<'a>) -> Self {
        Self {
            ast: p.ast,
            classes: p.classes,
            ids: p.ids,
        }
    }
}

impl<'a> VDom<'a> {
    pub fn get_element_by_id<S: ?Sized>(&'a self, id: &S) -> Option<Rc<Node<'a>>>
    where
        S: AsBytes,
    {
        let id = id.as_bytes();

        self.ids.get(&id).cloned()
    }
}
