use std::{borrow::Cow, collections::HashMap, rc::Rc};
use crate::Bytes;

/// Stores all attributes of an HTML tag, as well as additional metadata such as `id` and `class`
#[derive(Debug, Clone)]
pub struct Attributes<'a> {
    /// Raw attributes (maps attribute key to attribute value)
    pub raw: HashMap<Bytes<'a>, Option<Bytes<'a>>>,
    /// The ID of this HTML element, if present
    pub id: Option<Bytes<'a>>,
    /// A list of class names of this HTML element, if present
    pub class: Option<Bytes<'a>>,
}

impl<'a> Attributes<'a> {
    /// Creates a new `Attributes
    pub(crate) fn new() -> Self {
        Self {
            raw: HashMap::new(),
            id: None,
            class: None,
        }
    }
}

/// Represents a single HTML element
#[derive(Debug, Clone)]
pub struct HTMLTag<'a> {
    pub(crate) _name: Option<Bytes<'a>>,
    pub(crate) _attributes: Attributes<'a>,
    pub(crate) _children: Vec<Rc<Node<'a>>>,
    pub(crate) _raw: Bytes<'a>,
}

impl<'a> HTMLTag<'a> {
    /// Creates a new HTMLTag
    pub(crate) fn new(
        name: Option<Bytes<'a>>,
        attr: Attributes<'a>,
        children: Vec<Rc<Node<'a>>>,
        raw: Bytes<'a>,
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children,
            _raw: raw,
        }
    }

    /// Returns the name of this HTML tag
    pub fn name(&self) -> &Option<Bytes<'a>> {
        &self._name
    }

    /// Returns attributes of this HTML tag
    pub fn attributes(&self) -> &Attributes<'a> {
        &self._attributes
    }

    /// Returns the contained markup
    /// Equivalent to [Element#innerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML) in browsers)
    pub fn inner_html(&self) -> &Bytes<'a> {
        &self._raw
    }

    /// Returns the contained text of this element, excluding any markup
    /// Equivalent to [Element#innerText](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerText) in browsers)
    /// This function may not allocate memory for a new string as it can just return the part of the tag that doesn't have markup
    /// For tags that *do* have more than one subnode, this will allocate memory
    pub fn inner_text(&self) -> Cow<'a, str> {
        let len = self._children.len();

        if len == 0 {
            // If there are no subnodes, we can just return a static, empty, string slice
            return Cow::Borrowed("");
        }

        let first = &self._children[0];

        if len == 1 {
            match &**first {
                Node::Tag(t) => return t.inner_text(),
                Node::Raw(e) => return e.as_utf8_str(),
                Node::Comment(_) => return Cow::Borrowed(""),
            }
        }

        // If there are >1 nodes, we need to allocate a new string and push each inner_text in it
        // TODO: check if String::with_capacity() is worth it
        let mut s = String::from(first.inner_text());

        for node in self._children.iter().skip(1) {
            match &**node {
                Node::Tag(t) => s.push_str(&t.inner_text()),
                Node::Raw(e) => s.push_str(&e.as_utf8_str()),
                Node::Comment(_) => { /* no op */ }
            }
        }

        Cow::Owned(s)
    }
}

/// An HTML Node
#[derive(Debug, Clone)]
pub enum Node<'a> {
    /// A regular HTML element/tag
    Tag(HTMLTag<'a>),
    /// Raw text (no particular HTML element)
    Raw(Bytes<'a>),
    /// Comment (<!-- -->)
    Comment(Bytes<'a>),
}

impl<'a> Node<'a> {
    /// Returns the inner text of this node
    pub fn inner_text(&self) -> Cow<'a, str> {
        match self {
            Node::Comment(_) => Cow::Borrowed(""),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => t.inner_text(),
        }
    }
}