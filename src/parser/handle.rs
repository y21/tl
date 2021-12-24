use crate::Node;

use super::Parser;

/// The inner type of a NodeHandle, used to represent an index into the tags table
pub type InnerNodeHandle = usize;

/// An external handle to a HTML node, originally obtained from a [Parser]
///
/// It contains an identifier that uniquely identifies an HTML node.
/// In particular, it is an index into the global HTML tag table managed by the [`Parser`].
/// To get a [`Node`] out of a [`NodeHandle`], call `NodeHandle::get()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NodeHandle(InnerNodeHandle);

impl NodeHandle {
    /// Creates a new handle to the given node
    #[inline]
    pub fn new(node: InnerNodeHandle) -> Self {
        NodeHandle(node)
    }

    /// Returns a reference to the node that is associated to this specific handle
    pub fn get<'p, 'buf>(&self, parser: &'p Parser<'buf>) -> Option<&'p Node<'buf>> {
        parser.resolve_node_id(self.0)
    }

    /// Returns a mutable reference to the node that is associated to this specific handle
    pub fn get_mut<'p, 'buf>(&self, parser: &'p mut Parser<'buf>) -> Option<&'p mut Node<'buf>> {
        parser.resolve_node_id_mut(self.0)
    }

    /// Returns the internal unique Node ID that maps to a specific node in the node table
    #[inline]
    pub fn get_inner(&self) -> InnerNodeHandle {
        self.0
    }
}
