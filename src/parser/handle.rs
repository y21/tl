use crate::Node;

use super::Parser;

/// An external handle to a HTML node, originally obtained from a [Parser]
///
/// It contains an identifier that uniquely identifies an HTML node.
/// In particular, it is an index into the global HTML tag table managed by the [`Parser`].
/// To get a [`Node`] out of a [`NodeHandle`], call `NodeHandle::get()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NodeHandle(usize);

impl NodeHandle {
    /// Creates a new handle to the given node
    #[inline]
    pub fn new(node: usize) -> Self {
        NodeHandle(node)
    }

    /// Returns the node that is associated to this specific handle
    pub fn get<'p, 'buf>(&self, parser: &'p Parser<'buf>) -> Option<&'p Node<'buf>> {
        parser.resolve_node_id(self.0)
    }

    /// Returns the internal unique Node ID that maps to a specific node in the node table
    #[inline]
    pub fn get_inner(&self) -> usize {
        self.0
    }
}
