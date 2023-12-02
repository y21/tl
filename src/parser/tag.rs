use crate::{
    inline::{hashmap::InlineHashMap, vec::InlineVec},
    queryselector::{self, QuerySelectorIterator},
    Bytes, InnerNodeHandle,
};
use std::{borrow::Cow, mem};

use super::{handle::NodeHandle, Parser};

const INLINED_ATTRIBUTES: usize = 2;
const INLINED_SUBNODES: usize = 2;
const HTML_VOID_ELEMENTS: [&str; 16] = [
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];

/// The type of map for "raw" attributes
pub type RawAttributesMap<'a> = InlineHashMap<Bytes<'a>, Option<Bytes<'a>>, INLINED_ATTRIBUTES>;

/// The type of vector for children of an HTML tag
pub type RawChildren = InlineVec<NodeHandle, INLINED_SUBNODES>;

/// Stores all attributes of an HTML tag, as well as additional metadata such as `id` and `class`
#[derive(Debug, Clone)]
pub struct Attributes<'a> {
    /// Raw attributes (maps attribute key to attribute value)
    pub(crate) raw: RawAttributesMap<'a>,
    /// The ID of this HTML element, if present
    pub(crate) id: Option<Bytes<'a>>,
    /// A list of class names of this HTML element, if present
    pub(crate) class: Option<Bytes<'a>>,
}

impl<'a> Attributes<'a> {
    /// Creates a new `Attributes
    pub(crate) fn new() -> Self {
        Self {
            raw: InlineHashMap::new(),
            id: None,
            class: None,
        }
    }

    /// Counts the number of attributes
    pub fn len(&self) -> usize {
        let mut raw = self.raw.len();
        if self.id.is_some() {
            raw += 1;
        }
        if self.class.is_some() {
            raw += 1;
        }
        raw
    }

    /// Checks whether this collection of attributes is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks whether a given string is in the class names list
    pub fn is_class_member<B: AsRef<[u8]>>(&self, member: B) -> bool {
        self.class_iter()
            .map_or(false, |mut i| i.any(|s| s.as_bytes() == member.as_ref()))
    }

    /// Checks whether this attributes collection contains a given key and returns its value
    ///
    /// Attributes that exist in this tag but have no value set will have their inner Option set to None
    pub fn get<B>(&self, key: B) -> Option<Option<&Bytes<'a>>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.as_ref().map(Some),
            b"class" => self.class.as_ref().map(Some),
            _ => self.raw.get(&key).map(|x| x.as_ref()),
        }
    }

    /// Checks whether this attributes collection contains a given key
    pub fn contains<B>(&self, key: B) -> bool
    where
        B: Into<Bytes<'a>>,
    {
        self.get(key).is_some()
    }

    /// Removes an attribute from this collection and returns it.
    ///
    /// As with [`Attributes::get()`], the outer Option is set to None if the attribute does not exist.
    /// The inner option is set to None if the attribute exists but has no value.
    ///
    /// # Example
    /// ```
    /// let mut dom = tl::parse("<span contenteditable=\"true\"></span>", Default::default()).unwrap();
    /// let element = dom.nodes_mut()[0].as_tag_mut().unwrap();
    /// let attributes = element.attributes_mut();
    ///
    /// assert_eq!(attributes.remove("contenteditable"), Some(Some("true".into())));
    /// assert_eq!(attributes.len(), 0);
    /// ```
    pub fn remove<B>(&mut self, key: B) -> Option<Option<Bytes<'a>>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.take().map(Some),
            b"class" => self.class.take().map(Some),
            _ => self.raw.remove(&key),
        }
    }

    /// Removes the value of an attribute in this collection and returns it.
    ///
    /// # Example
    /// ```
    /// let mut dom = tl::parse("<span contenteditable=\"true\"></span>", Default::default()).unwrap();
    /// let element = dom.nodes_mut()[0].as_tag_mut().unwrap();
    /// let attributes = element.attributes_mut();
    ///
    /// assert_eq!(attributes.remove_value("contenteditable"), Some("true".into()));
    /// assert_eq!(attributes.get("contenteditable"), Some(None));
    /// ```
    pub fn remove_value<B>(&mut self, key: B) -> Option<Bytes<'a>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.take(),
            b"class" => self.class.take(),
            _ => self.raw.get_mut(&key).and_then(mem::take),
        }
    }

    /// Checks whether this attributes collection contains a given key and returns its value
    pub fn get_mut<B>(&mut self, key: B) -> Option<Option<&mut Bytes<'a>>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.as_mut().map(Some),
            b"class" => self.class.as_mut().map(Some),
            _ => self.raw.get_mut(&key).map(Option::as_mut),
        }
    }

    /// Inserts a new attribute into this attributes collection
    pub fn insert<K, V>(&mut self, key: K, value: Option<V>)
    where
        K: Into<Bytes<'a>>,
        V: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();
        let value = value.map(Into::into);

        match key.as_bytes() {
            b"id" => self.id = value,
            b"class" => self.class = value,
            _ => self.raw.insert(key, value),
        };
    }

    /// Returns an iterator `(attribute_key, attribute_value)` over the attributes of this `HTMLTag`
    pub fn iter(&self) -> impl Iterator<Item = (Cow<str>, Option<Cow<str>>)> + '_ {
        self.raw
            .iter()
            .map(|(k, v)| {
                let k = k.as_utf8_str();
                let v = v.as_ref().map(|x| x.as_utf8_str());

                (Some(k), v)
            })
            .chain([
                (
                    self.id.is_some().then(|| Cow::Borrowed("id")),
                    self.id.as_ref().map(|x| x.as_utf8_str()),
                ),
                (
                    self.class.is_some().then(|| Cow::Borrowed("class")),
                    self.class.as_ref().map(|x| x.as_utf8_str()),
                ),
            ])
            .flat_map(|(k, v)| k.map(|k| (k, v)))
    }

    /// Returns the `id` attribute of this HTML tag, if present
    pub fn id(&self) -> Option<&Bytes<'a>> {
        self.id.as_ref()
    }

    /// Returns the `class` attribute of this HTML tag, if present
    pub fn class(&self) -> Option<&Bytes<'a>> {
        self.class.as_ref()
    }

    /// Returns an iterator over all of the class members
    pub fn class_iter(&self) -> Option<impl Iterator<Item = &'_ str> + '_> {
        self.class
            .as_ref()
            .and_then(Bytes::try_as_utf8_str)
            .map(str::split_ascii_whitespace)
    }

    /// Returns the underlying raw map for attributes
    ///
    /// ## A note on stability
    /// It is not guaranteed for the returned map to include all attributes.
    /// Some attributes may be stored in `Attributes` itself and not in the raw map.
    /// For that reason you should prefer to call methods on `Attributes` directly,
    /// i.e. `Attributes::get()` to lookup an attribute by its key.
    pub fn unstable_raw(&self) -> &RawAttributesMap<'a> {
        &self.raw
    }
}

/// Represents a single HTML element
#[derive(Debug, Clone)]
pub struct HTMLTag<'a> {
    pub(crate) _name: Bytes<'a>,
    pub(crate) _attributes: Attributes<'a>,
    pub(crate) _children: RawChildren,
    pub(crate) _raw: Bytes<'a>,
}

impl<'a> HTMLTag<'a> {
    /// Creates a new HTMLTag
    #[inline(always)]
    pub(crate) fn new(
        name: Bytes<'a>,
        attr: Attributes<'a>,
        children: InlineVec<NodeHandle, INLINED_SUBNODES>,
        raw: Bytes<'a>,
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children,
            _raw: raw,
        }
    }

    /// Returns a wrapper around the children of this HTML tag
    #[inline]
    pub fn children(&self) -> Children<'a, '_> {
        Children(self)
    }

    /// Returns a mutable wrapper around the children of this HTML tag.
    pub fn children_mut(&mut self) -> ChildrenMut<'a, '_> {
        ChildrenMut(self)
    }

    /// Returns the name of this HTML tag
    #[inline]
    pub fn name(&self) -> &Bytes<'a> {
        &self._name
    }

    /// Returns a mutable reference to the name of this HTML tag
    #[inline]
    pub fn name_mut(&mut self) -> &mut Bytes<'a> {
        &mut self._name
    }

    /// Returns attributes of this HTML tag
    #[inline]
    pub fn attributes(&self) -> &Attributes<'a> {
        &self._attributes
    }

    /// Returns a mutable reference to the attributes of this HTML tag
    #[inline]
    pub fn attributes_mut(&mut self) -> &mut Attributes<'a> {
        &mut self._attributes
    }

    /// Returns the contained markup
    ///
    /// ## Limitations
    /// - The order of tag attributes is not guaranteed
    /// - Spaces within the tag are not preserved (i.e. `<img      src="">` may become `<img src="">`)
    ///
    /// Equivalent to [Element#outerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/outerHTML) in browsers)
    pub fn outer_html<'p>(&'p self, parser: &'p Parser<'a>) -> String {
        let tag_name = self._name.as_utf8_str();
        let is_void_element = HTML_VOID_ELEMENTS.contains(&tag_name.as_ref());
        let mut outer_html = format!("<{}", &tag_name);

        #[inline]
        fn write_attribute(dest: &mut String, k: Cow<str>, v: Option<Cow<str>>) {
            dest.push(' ');

            dest.push_str(&k);

            if let Some(value) = v {
                dest.push_str("=\"");
                dest.push_str(&value);
                dest.push('"');
            }
        }

        let attr = self.attributes();

        for (k, v) in attr.iter() {
            write_attribute(&mut outer_html, k, v);
        }

        outer_html.push('>');

        // void elements have neither content nor a closing tag.
        if is_void_element {
            return outer_html;
        }

        // TODO(y21): More of an idea than a TODO, but a potential perf improvement
        // could be having some kind of internal inner_html function that takes a &mut String
        // and simply writes to it instead of returning a newly allocated string for every element
        // and appending it
        outer_html.push_str(&self.inner_html(parser));

        outer_html.push_str("</");
        outer_html.push_str(&self._name.as_utf8_str());
        outer_html.push('>');

        outer_html
    }

    /// Returns the contained markup
    ///
    /// ## Limitations
    /// - The order of tag attributes is not guaranteed
    /// - Spaces within the tag are not preserved (i.e. `<img      src="">` may become `<img src="">`)
    ///
    /// Equivalent to [Element#innerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML) in browsers)
    pub fn inner_html<'p>(&'p self, parser: &'p Parser<'a>) -> String {
        self.children()
            .top()
            .iter()
            .map(|handle| handle.get(parser).unwrap())
            .map(|node| node.outer_html(parser))
            .collect::<String>()
    }

    /// Returns the raw HTML of this tag.
    /// This is a cheaper version of `HTMLTag::inner_html` if you never mutate any nodes.
    ///
    /// **Note:** Mutating this tag does *not* re-compute the HTML representation of this tag.
    /// This simply returns a reference to the substring.
    pub fn raw(&self) -> &Bytes<'a> {
        &self._raw
    }

    /// Returns the boundaries/position `(start, end)` of this HTML tag in the source string.
    ///
    /// # Example
    /// ```
    /// let source = "<p><span>hello</span></p>";
    /// let dom = tl::parse(source, Default::default()).unwrap();
    /// let parser = dom.parser();
    /// let span = dom.nodes().iter().filter_map(|n| n.as_tag()).find(|n| n.name() == "span").unwrap();
    /// let (start, end) = span.boundaries(parser);
    /// assert_eq!((start, end), (3, 20));
    /// assert_eq!(&source[start..=end], "<span>hello</span>");
    /// ```
    pub fn boundaries(&self, parser: &Parser<'a>) -> (usize, usize) {
        let raw = self._raw.as_bytes();
        let input = parser.stream.data().as_ptr();
        let start = raw.as_ptr();
        let offset = start as usize - input as usize;
        let end = offset + raw.len() - 1;
        (offset, end)
    }

    /// Returns the contained text of this element, excluding any markup.
    /// Equivalent to [Element#innerText](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerText) in browsers)
    /// This function may not allocate memory for a new string as it can just return the part of the tag that doesn't have markup.
    /// For tags that *do* have more than one subnode, this will allocate memory
    pub fn inner_text<'p>(&self, parser: &'p Parser<'a>) -> Cow<'p, str> {
        let len = self._children.len();

        if len == 0 {
            // If there are no subnodes, we can just return a static, empty, string slice
            return Cow::Borrowed("");
        }

        let first = self._children[0].get(parser).unwrap();

        if len == 1 {
            match &first {
                Node::Tag(t) => return t.inner_text(parser),
                Node::Raw(e) => return e.as_utf8_str(),
                Node::Comment(_) => return Cow::Borrowed(""),
            }
        }

        // If there are >1 nodes, we need to allocate a new string and push each inner_text in it
        // TODO: check if String::with_capacity() is worth it
        let mut s = String::from(first.inner_text(parser));

        for &id in self._children.iter().skip(1) {
            let node = id.get(parser).unwrap();

            match &node {
                Node::Tag(t) => s.push_str(&t.inner_text(parser)),
                Node::Raw(e) => s.push_str(&e.as_utf8_str()),
                Node::Comment(_) => { /* no op */ }
            }
        }

        Cow::Owned(s)
    }

    /// Tries to parse the query selector and returns an iterator over elements that match the given query selector.
    ///
    /// # Example
    /// ```
    /// let dom = tl::parse(r#"
    ///     <div class="x">
    ///     <div class="y">
    ///       <div class="z">MATCH</div>
    ///       <div class="z">MATCH</div>
    ///       <div class="z">MATCH</div>
    ///     </div>
    ///   </div>
    ///   <div class="z">NO MATCH</div>
    ///   <div class="z">NO MATCH</div>
    ///   <div class="z">NO MATCH</div>
    /// "#, Default::default()).unwrap();
    /// let parser = dom.parser();
    ///
    /// let outer = dom
    ///     .get_elements_by_class_name("y")
    ///     .next()
    ///     .unwrap()
    ///     .get(parser)
    ///     .unwrap()
    ///     .as_tag()
    ///     .unwrap();
    ///
    /// let inner_z = outer.query_selector(parser, ".z").unwrap();
    ///
    /// assert_eq!(inner_z.clone().count(), 3);
    ///
    /// for handle in inner_z {
    ///     let node = handle.get(parser).unwrap().as_tag().unwrap();
    ///     assert_eq!(node.inner_text(parser), "MATCH");
    /// }
    ///
    /// ```
    pub fn query_selector<'b>(
        &'b self,
        parser: &'b Parser<'a>,
        selector: &'b str,
    ) -> Option<QuerySelectorIterator<'a, 'b, Self>> {
        let selector = crate::parse_query_selector(selector)?;
        let iter = queryselector::QuerySelectorIterator::new(selector, parser, self);
        Some(iter)
    }

    /// Calls the given closure with each tag as parameter
    ///
    /// The closure must return a boolean, indicating whether it should stop iterating
    /// Returning `true` will break the loop
    pub fn find_node<F>(&self, parser: &Parser<'a>, f: &mut F) -> Option<NodeHandle>
    where
        F: FnMut(&Node<'a>) -> bool,
    {
        for &id in self._children.iter() {
            let node = id.get(parser).unwrap();

            if f(node) {
                return Some(id);
            }
        }
        None
    }
}

/// A thin wrapper around the children of [`HTMLTag`]
#[derive(Debug, Clone)]
pub struct Children<'a, 'b>(&'b HTMLTag<'a>);

impl<'a, 'b> Children<'a, 'b> {
    /// Returns the topmost, direct children of this tag.
    ///
    /// # Example
    /// ```
    /// let dom = tl::parse(r#"
    ///     <div id="a">
    ///         <div id="b">
    ///             <span>Hello</span>
    ///             <span>World</span>
    ///             <span>.</span>
    ///         </div>
    ///     </div>
    /// "#, Default::default()).unwrap();
    ///
    /// let a = dom.get_element_by_id("a")
    ///     .unwrap()
    ///     .get(dom.parser())
    ///     .unwrap()
    ///     .as_tag()
    ///     .unwrap();
    ///
    /// // Calling this function on the first div tag (#a) will return a slice containing 3 elements:
    /// // - whitespaces around (before and after) div#b
    /// // - div#b itself
    /// // It does **not** contain the inner span tags
    /// assert_eq!(a.children().top().len(), 3);
    /// ```
    #[inline]
    pub fn top(&self) -> &RawChildren {
        &self.0._children
    }

    /// Returns the starting boundary of the children of this tag.
    #[inline]
    pub fn start(&self) -> Option<InnerNodeHandle> {
        self.0._children.get(0).map(NodeHandle::get_inner)
    }

    /// Returns the ending boundary of the children of this tag.
    pub fn end(&self, parser: &Parser<'a>) -> Option<InnerNodeHandle> {
        find_last_node_handle(self.0, parser).map(|h| h.get_inner())
    }

    /// Returns the (start, end) boundaries of the children of this tag.
    #[inline]
    pub fn boundaries(&self, parser: &Parser<'a>) -> Option<(InnerNodeHandle, InnerNodeHandle)> {
        self.start().zip(self.end(parser))
    }

    /// Returns a slice containing all of the children of this [`HTMLTag`],
    /// including all subnodes of the children.
    ///
    /// The difference between `top()` and `all()` is the same as `VDom::children()` and `VDom::nodes()`
    ///
    /// # Example
    /// ```
    /// let dom = tl::parse(r#"
    ///     <div id="a"><div id="b"><span>Hello</span><span>World</span><span>!</span></div></div>
    /// "#, Default::default()).unwrap();
    ///
    /// let a = dom.get_element_by_id("a")
    ///     .unwrap()
    ///     .get(dom.parser())
    ///     .unwrap()
    ///     .as_tag()
    ///     .unwrap();
    ///
    /// // Calling this function on the first div tag (#a) will return a slice containing all of the subnodes:
    /// // - div#b
    /// // - span
    /// // - Hello
    /// // - span
    /// // - World
    /// // - span
    /// // - !
    /// assert_eq!(a.children().all(dom.parser()).len(), 7);
    /// ```
    pub fn all(&self, parser: &'b Parser<'a>) -> &'b [Node<'a>] {
        self.boundaries(parser)
            .map(|(start, end)| &parser.tags[start as usize..=end as usize])
            .unwrap_or(&[])
    }
}

/// A thin mutable wrapper around the children of [`HTMLTag`]
#[derive(Debug)]
pub struct ChildrenMut<'a, 'b>(&'b mut HTMLTag<'a>);

impl<'a, 'b> ChildrenMut<'a, 'b> {
    /// Returns the topmost, direct children of this tag as a mutable slice.
    ///
    /// See [`Children::top`] for more details and examples.
    #[inline]
    pub fn top_mut(&mut self) -> &mut RawChildren {
        &mut self.0._children
    }
}

/// Attempts to find the very last node handle that is contained in the given tag
fn find_last_node_handle<'a>(tag: &HTMLTag<'a>, parser: &Parser<'a>) -> Option<NodeHandle> {
    let mut tag = tag;
    let mut last_handle = None;

    loop {
        if let Some(last_child_handle) = tag._children.as_slice().last().copied() {
            last_handle = Some(last_child_handle);
            if let Some(child) = last_child_handle
                .get(parser)
                .expect("Failed to get child node, please open a bug report") // this shouldn't happen
                .as_tag()
            {
                tag = child; // Continue looking at the child
                continue;
            }
        };
        break last_handle;
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
    pub fn inner_text<'s, 'p: 's>(&'s self, parser: &'p Parser<'a>) -> Cow<'s, str> {
        match self {
            Node::Comment(_) => Cow::Borrowed(""),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => t.inner_text(parser),
        }
    }

    /// Returns the outer HTML of this node
    pub fn outer_html<'s>(&'s self, parser: &Parser<'a>) -> Cow<'s, str> {
        match self {
            Node::Comment(c) => c.as_utf8_str(),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => Cow::Owned(t.outer_html(parser)),
        }
    }

    /// Returns the inner HTML of this node
    pub fn inner_html<'s>(&'s self, parser: &Parser<'a>) -> Cow<'s, str> {
        match self {
            Node::Comment(c) => c.as_utf8_str(),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => Cow::Owned(t.inner_html(parser)),
        }
    }

    /// Returns an iterator over subnodes ("children") of this HTML tag, if this is a tag
    pub fn children(&self) -> Option<Children<'a, '_>> {
        match self {
            Node::Tag(t) => Some(t.children()),
            _ => None,
        }
    }

    /// Calls the given closure with each tag as parameter
    ///
    /// The closure must return a boolean, indicating whether it should stop iterating
    /// Returning `true` will break the loop and return a handle to the node
    pub fn find_node<F>(&self, parser: &Parser<'a>, f: &mut F) -> Option<NodeHandle>
    where
        F: FnMut(&Node<'a>) -> bool,
    {
        if let Some(children) = self.children() {
            for &id in children.top().iter() {
                let node = id.get(parser).unwrap();

                if f(node) {
                    return Some(id);
                }

                let subnode = node.find_node(parser, f);
                if subnode.is_some() {
                    return subnode;
                }
            }
        }
        None
    }

    /// Tries to coerce this node into a `HTMLTag` variant
    pub fn as_tag(&self) -> Option<&HTMLTag<'a>> {
        match self {
            Self::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    /// Tries to coerce this node into a `HTMLTag` variant
    pub fn as_tag_mut(&mut self) -> Option<&mut HTMLTag<'a>> {
        match self {
            Self::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    /// Tries to coerce this node into a comment, returning the text
    pub fn as_comment(&self) -> Option<&Bytes<'a>> {
        match self {
            Self::Comment(c) => Some(c),
            _ => None,
        }
    }

    /// Tries to coerce this node into a comment, returning the text
    pub fn as_comment_mut(&mut self) -> Option<&mut Bytes<'a>> {
        match self {
            Self::Comment(c) => Some(c),
            _ => None,
        }
    }

    /// Tries to coerce this node into a raw text node, returning the text
    ///
    /// "Raw text nodes" are nodes that are not HTML tags, but just text
    pub fn as_raw(&self) -> Option<&Bytes<'a>> {
        match self {
            Self::Raw(r) => Some(r),
            _ => None,
        }
    }

    /// Tries to coerce this node into a mutable raw text node, returning the text
    ///
    /// "Raw text nodes" are nodes that are not HTML tags, but just text
    pub fn as_raw_mut(&mut self) -> Option<&mut Bytes<'a>> {
        match self {
            Self::Raw(r) => Some(r),
            _ => None,
        }
    }
}
