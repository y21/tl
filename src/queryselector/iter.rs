use crate::{NodeHandle, VDom};

use super::Selector;

pub struct QuerySelectorIterator<'a, 'b> {
    selector: Selector<'b>,
    dom: &'b VDom<'a>,
    index: usize,
}

impl<'a, 'b> QuerySelectorIterator<'a, 'b> {
    pub fn new(selector: Selector<'b>, dom: &'b VDom<'a>) -> Self {
        Self {
            selector,
            dom,
            index: 0,
        }
    }
}

impl<'a, 'b> Iterator for QuerySelectorIterator<'a, 'b> {
    type Item = NodeHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let parser = self.dom.parser();
        let nodes = parser.tags.iter().enumerate().skip(self.index);

        for (idx, node) in nodes {
            self.index += 1;
            let matches = self.selector.matches(self.dom, node);

            if matches {
                return Some(NodeHandle::new(idx));
            }
        }

        None
    }
}
