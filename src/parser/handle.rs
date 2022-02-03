use crate::Node;

use super::Parser;

/// The inner type of a NodeHandle, used to represent an index into the tags table
pub type InnerNodeHandle = u32;

/// A detached, external handle to a HTML node, originally obtained from a [Parser]
///
/// It contains an identifier that uniquely identifies an HTML node.
/// In particular, it is an index into the global HTML tag table managed by the [`Parser`].
/// To get a [`Node`] out of a [`NodeHandle`], call `NodeHandle::get()`
///
/// A common way to model self referential/recursive graphs is to have one "global" vector
/// of nodes, and store indices into the vector instead of references.
/// In the case of tl, the "global" HTML tag vector is stored in the [`Parser`] and [`NodeHandle`] represents the index.
/// Because [`NodeHandle`] is only an index and completely detached from anything, you need to pass a parser to `NodeHandle::get()`
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
    ///
    /// It is an error to pass in the wrong parser.
    /// It will either return `None` if this index points outside of the nodes table,
    /// or it will return the one it points to.
    pub fn get<'p, 'buf>(&self, parser: &'p Parser<'buf>) -> Option<&'p Node<'buf>> {
        parser.resolve_node_id(self.0)
    }

    /// Returns a mutable reference to the node that is associated to this specific handle
    ///
    /// It is an error to pass in the wrong parser.
    /// It will either return `None` if this index points outside of the nodes table,
    /// or it will return the one it points to.
    pub fn get_mut<'p, 'buf>(&self, parser: &'p mut Parser<'buf>) -> Option<&'p mut Node<'buf>> {
        parser.resolve_node_id_mut(self.0)
    }

    /// Returns the internal unique Node ID that maps to a specific node in the node table
    #[inline]
    pub fn get_inner(&self) -> InnerNodeHandle {
        self.0
    }
}
