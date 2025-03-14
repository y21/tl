use crate::errors::ParseError;
use crate::parser::HTMLVersion;
use crate::parser::NodeHandle;
use crate::queryselector;
use crate::queryselector::QuerySelectorIterator;
use crate::Bytes;
use crate::InnerNodeHandle;
use crate::ParserOptions;
use crate::{Node, Parser};
use std::marker::PhantomData;

/// VDom represents a [Document Object Model](https://developer.mozilla.org/en/docs/Web/API/Document_Object_Model)
///
/// It is the result of parsing an HTML document.
/// Internally it is only a wrapper around the [`Parser`] struct, in which all of the HTML tags are stored.
/// Many functions of the public API take a reference to a [`Parser`] as a parameter to resolve [`NodeHandle`]s to [`Node`]s.
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
    /// Returns a reference to the underlying parser
    #[inline]
    pub fn parser(&self) -> &Parser<'a> {
        &self.parser
    }

    /// Returns a mutable reference to the underlying parser
    #[inline]
    pub fn parser_mut(&mut self) -> &mut Parser<'a> {
        &mut self.parser
    }

    /// Finds an element by its `id` attribute.
    pub fn get_element_by_id<'b, S>(&'b self, id: S) -> Option<NodeHandle>
    where
        S: Into<Bytes<'a>>,
    {
        let bytes: Bytes = id.into();
        let parser = self.parser();

        if parser.options.is_tracking_ids() {
            parser.ids.get(&bytes).copied()
        } else {
            self.nodes()
                .iter()
                .enumerate()
                .find(|(_, node)| {
                    node.as_tag().map_or(false, |tag| {
                        tag._attributes.id.as_ref().map_or(false, |x| x.eq(&bytes))
                    })
                })
                .map(|(id, _)| NodeHandle::new(id as InnerNodeHandle))
        }
    }

    /// Returns a list of elements that match a given class name.
    pub fn get_elements_by_class_name<'b>(
        &'b self,
        id: &'b str,
    ) -> Box<dyn Iterator<Item = NodeHandle> + 'b> {
        let parser = self.parser();

        if parser.options.is_tracking_classes() {
            parser
                .classes
                .get(&Bytes::from(id.as_bytes()))
                .map(|x| Box::new(x.iter().cloned()) as Box<dyn Iterator<Item = NodeHandle>>)
                .unwrap_or_else(|| Box::new(std::iter::empty()))
        } else {
            let member = id;

            let iter = self
                .nodes()
                .iter()
                .enumerate()
                .filter_map(move |(id, node)| {
                    node.as_tag().and_then(|tag| {
                        tag._attributes
                            .is_class_member(member)
                            .then(|| NodeHandle::new(id as InnerNodeHandle))
                    })
                });

            Box::new(iter)
        }
    }

    /// Returns a slice of *all* the elements in the HTML document
    ///
    /// The difference between `children()` and `nodes()` is that children only returns the immediate children of the root node,
    /// while `nodes()` returns all nodes, including nested tags.
    ///
    /// # Order
    /// The order of the returned nodes is the same as the order of the nodes in the HTML document.
    pub fn nodes(&self) -> &[Node<'a>] {
        &self.parser.tags
    }

    /// Returns a mutable slice of *all* the elements in the HTML document
    ///
    /// The difference between `children()` and `nodes()` is that children only returns the immediate children of the root node,
    /// while `nodes()` returns all nodes, including nested tags.
    pub fn nodes_mut(&mut self) -> &mut [Node<'a>] {
        &mut self.parser.tags
    }

    /// Returns the topmost subnodes ("children") of this DOM
    pub fn children(&self) -> &[NodeHandle] {
        &self.parser.ast
    }

    /// Returns a mutable reference to the topmost subnodes ("children") of this DOM
    pub fn children_mut(&mut self) -> &mut [NodeHandle] {
        &mut self.parser.ast
    }

    /// Returns the HTML version.
    /// This is determined by the `<!DOCTYPE>` tag
    pub fn version(&self) -> Option<HTMLVersion> {
        self.parser.version
    }

    /// Returns the contained markup of all of the elements in this DOM.
    ///
    /// Equivalent to [Element#outerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/outerHTML) in browsers)
    ///
    /// # Example
    /// ```
    /// let html = r#"<div><p href="/about" id="find-me">Hello world</p></div>"#;
    /// let mut dom = tl::parse(html, Default::default()).unwrap();
    ///
    /// let element = dom.get_element_by_id("find-me")
    ///     .unwrap()
    ///     .get_mut(dom.parser_mut())
    ///     .unwrap()
    ///     .as_tag_mut()
    ///     .unwrap();
    ///
    /// element.attributes_mut().get_mut("href").flatten().unwrap().set("/");
    ///
    /// assert_eq!(dom.outer_html(), r#"<div><p href="/" id="find-me">Hello world</p></div>"#);
    /// ```
    pub fn outer_html(&self) -> String {
        let mut inner_html = String::with_capacity(self.parser.stream.len());

        for node in self.children() {
            let node = node.get(&self.parser).unwrap();
            inner_html.push_str(&node.outer_html(&self.parser));
        }

        inner_html
    }

    /// Tries to parse the query selector and returns an iterator over elements that match the given query selector.
    ///
    /// # Example
    /// ```
    /// let dom = tl::parse("<div><p class=\"foo\">bar</div>", tl::ParserOptions::default()).unwrap();
    /// let handle = dom.query_selector("p.foo").and_then(|mut iter| iter.next()).unwrap();
    /// let node = handle.get(dom.parser()).unwrap();
    /// assert_eq!(node.inner_text(dom.parser()), "bar");
    /// ```
    pub fn query_selector<'b>(
        &'b self,
        selector: &'b str,
    ) -> Option<QuerySelectorIterator<'a, 'b, Self>> {
        let selector = crate::parse_query_selector(selector)?;
        let iter = queryselector::QuerySelectorIterator::new(selector, self.parser(), self);
        Some(iter)
    }
}

/// A RAII guarded version of VDom
///
/// The input string is freed once this struct goes out of scope.
/// The only way to construct this is by calling `parse_owned()`.
#[derive(Debug)]
pub struct VDomGuard {
    /// Wrapped VDom instance
    dom: VDom<'static>,
    /// The leaked input string that is referenced by self.dom
    _s: RawString,
    /// PhantomData for self.dom
    _phantom: PhantomData<&'static str>,
}

unsafe impl Send for VDomGuard {}
unsafe impl Sync for VDomGuard {}

impl VDomGuard {
    /// Parses the input string
    pub(crate) fn parse(input: String, options: ParserOptions) -> Result<VDomGuard, ParseError> {
        let input = RawString::new(input);

        let ptr = input.as_ptr();

        let input_ref: &'static str = unsafe { &*ptr };

        // Parsing will either:
        // a) succeed, and we return a VDom instance
        //    that, when dropped, will free the input string
        // b) fail, and we return a ParseError
        //    and `RawString`s destructor will run and deallocate the string properly
        let mut parser = Parser::new(input_ref, options);
        parser.parse()?;

        Ok(Self {
            _s: input,
            dom: VDom::from(parser),
            _phantom: PhantomData,
        })
    }
}

impl VDomGuard {
    /// Returns a reference to the inner DOM.
    ///
    /// The lifetime of the returned `VDom` is bound to self so that elements cannot outlive this `VDomGuard` struct.
    pub fn get_ref<'a>(&'a self) -> &'a VDom<'a> {
        &self.dom
    }

    /// Returns a mutable reference to the inner DOM.
    ///
    /// The lifetime of the returned `VDom` is bound to self so that elements cannot outlive this `VDomGuard` struct.
    pub fn get_mut_ref(&mut self) -> &mut VDom<'static>
    {
        &mut self.dom
    }
}

#[derive(Debug)]
struct RawString(*mut str);

impl RawString {
    pub fn new(s: String) -> Self {
        Self(Box::into_raw(s.into_boxed_str()))
    }

    pub fn as_ptr(&self) -> *mut str {
        self.0
    }
}

impl Drop for RawString {
    fn drop(&mut self) {
        // SAFETY: the pointer is always valid because `RawString` can only be constructed through `RawString::new()`
        unsafe {
            drop(Box::from_raw(self.0));
        };
    }
}
