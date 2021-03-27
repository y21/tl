use crate::{Node, Parser, Tree};
use crate::{bytes::AsBytes, parser::HTMLVersion};
use std::{marker::PhantomData, rc::Rc};

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

/// A RAII guarded version of VDom
///
/// The input string is freed once this struct goes out of scope.
/// The only way to construct this is by calling `parse_owned()`.
#[derive(Debug)]
pub struct VDomGuard<'a> {
    /// Pointer to leaked input string
    ptr: *mut str,
    /// Wrapped VDom instance
    dom: VDom<'a>,
    /// PhantomData for self.dom
    _phantom: PhantomData<&'a str>,
}

// SAFETY: The string is leaked and pinned to a memory location
unsafe impl<'a> Send for VDomGuard<'a> {}
unsafe impl<'a> Sync for VDomGuard<'a> {}

impl<'a> VDomGuard<'a> {
    /// Parses the input string
    pub(crate) fn parse(input: String) -> VDomGuard<'a> {
        let ptr = Box::into_raw(input.into_boxed_str());

        // SAFETY: Shortening the lifetime of the input string is fine, as it's `'static`
        let input_extended: &'a str = unsafe { &*ptr };

        let parser = Parser::new(input_extended).parse();

        Self {
            ptr,
            dom: VDom::from(parser),
            _phantom: PhantomData,
        }
    }
}

impl<'a> VDomGuard<'a> {
    /// Returns a reference to the inner DOM.
    ///
    /// The lifetime of `VDOM` is bound to self so that elements cannot outlive this `VDomGuard` struct.
    pub fn get_ref<'b>(&'a self) -> &'b VDom<'a> {
        &self.dom
    }
}

impl<'a> Drop for VDomGuard<'a> {
    fn drop(&mut self) {
        // SAFETY: We made this pointer in VDomGuard::parse() so we know it is properly aligned and non-null
        drop(unsafe { Box::from_raw(self.ptr) });
    }
}
