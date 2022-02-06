use crate::{HTMLTag, InnerNodeHandle, Node, NodeHandle, Parser, VDom};

mod private {
    pub trait Sealed {}
}

/// Trait for types that a query selector can iterate over
pub trait QueryIterable<'a>: private::Sealed {
    /// Gets a node at a specific index
    fn get<'b>(
        &'b self,
        parser: &'b Parser<'a>,
        index: usize,
    ) -> Option<(&'b Node<'a>, NodeHandle)>;
    /// Gets or computes the length (number of nodes)
    fn len(&self, parser: &Parser) -> usize;
    /// Gets the starting index
    fn start(&self) -> Option<InnerNodeHandle>;
}

impl<'a> private::Sealed for VDom<'a> {}
impl<'a> QueryIterable<'a> for VDom<'a> {
    #[inline]
    fn get<'b>(
        &'b self,
        parser: &'b Parser<'a>,
        index: usize,
    ) -> Option<(&'b Node<'a>, NodeHandle)> {
        // In a VDom, the index is equal to the node's id
        // and as such, we can recreate a `NodeHandle` from that ID
        parser
            .tags
            .get(index)
            .map(|node| (node, NodeHandle::new(index as u32)))
    }

    #[inline]
    fn len(&self, _parser: &Parser) -> usize {
        self.parser().tags.len()
    }

    #[inline]
    fn start(&self) -> Option<InnerNodeHandle> {
        // The starting ID is always 0 in a VDom
        Some(0)
    }
}

impl<'a> private::Sealed for HTMLTag<'a> {}
impl<'a> QueryIterable<'a> for HTMLTag<'a> {
    #[inline]
    fn get<'b>(
        &'b self,
        parser: &'b Parser<'a>,
        index: usize,
    ) -> Option<(&'b Node<'a>, NodeHandle)> {
        // Add `index` to the starting ID to get the ID of the node we need
        let index = self.start().map(|h| h as usize + index)?;
        let handle = NodeHandle::new(index as u32);
        let node = parser.tags.get(index)?;
        Some((node, handle))
    }

    #[inline]
    fn len(&self, parser: &Parser) -> usize {
        if let Some((start, end)) = self.children().boundaries(parser) {
            ((end - start) + 1) as usize
        } else {
            0
        }
    }

    #[inline]
    fn start(&self) -> Option<InnerNodeHandle> {
        self.children().start()
    }
}
