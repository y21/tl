use super::{
    constants,
    handle::NodeHandle,
    tag::{Attributes, HTMLTag, Node},
};
use crate::stream::Stream;
use crate::util;
use crate::{bytes::Bytes, inline::vec::InlineVec};
use std::collections::HashMap;

/// A list of HTML nodes
pub type Tree<'a> = Vec<Node<'a>>;

pub type ClassVec = InlineVec<NodeHandle, 2>;

/// HTML Version (<!DOCTYPE>)
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HTMLVersion {
    /// HTML Version 5
    HTML5,
    /// Strict HTML 4.01
    StrictHTML401,
    /// Transitional HTML 4.01
    TransitionalHTML401,
    /// Frameset HTML 4.01:
    FramesetHTML401,
}

/// The main HTML parser
///
/// Users of this library are not supposed to directly construct this struct.
/// Instead, users must call `tl::parse()` and use the returned `VDom`.
#[derive(Debug)]
pub struct Parser<'a> {
    /// A global collection of all HTML tags that appear in the source code
    ///
    /// HTML Nodes contain indicies into this vector
    pub(crate) tags: Tree<'a>,
    /// The inner stream that is used to iterate through the HTML source
    pub(crate) stream: Stream<'a, u8>,
    /// The topmost HTML nodes
    pub(crate) ast: Vec<NodeHandle>,
    /// A HashMap that maps Tag ID to a Node ID
    pub(crate) ids: HashMap<Bytes<'a>, NodeHandle>,
    /// A HashMap that maps Tag Class to a Node ID
    pub(crate) classes: HashMap<Bytes<'a>, ClassVec>,
    /// The current HTML version, if set
    pub(crate) version: Option<HTMLVersion>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &str) -> Parser {
        Parser {
            tags: Vec::new(),
            stream: Stream::new(input.as_bytes()),
            ast: Vec::new(),
            ids: HashMap::new(),
            classes: HashMap::new(),
            version: None,
        }
    }

    fn register_tag(&mut self, node: Node<'a>) -> NodeHandle {
        self.tags.push(node);
        NodeHandle::new(self.tags.len() - 1)
    }

    fn skip_whitespaces(&mut self) {
        self.read_while(&[b' ', b'\n']);
    }

    fn read_to(&mut self, terminator: &[u8]) -> &'a [u8] {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let ch = self.stream.current_unchecked();

            let end = self.stream.idx;

            if terminator.contains(ch) {
                return self.stream.slice_unchecked(start, end);
            }

            self.stream.advance();
        }

        self.stream.slice_unchecked(start, self.stream.idx)
    }

    fn read_while(&mut self, terminator: &[u8]) {
        while !self.stream.is_eof() {
            let ch = self.stream.current_unchecked();

            if !terminator.contains(ch) {
                break;
            }

            self.stream.advance();
        }
    }

    fn read_ident(&mut self) -> Option<&'a [u8]> {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let ch = self.stream.current_cpy()?;

            if !util::is_ident(ch) {
                let idx = self.stream.idx;
                return Some(self.stream.slice_unchecked(start, idx));
            }

            self.stream.advance();
        }

        None
    }

    fn skip_comment(&mut self) -> Option<&'a [u8]> {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let idx = self.stream.idx;

            if self
                .stream
                .slice_len(idx, constants::COMMENT.len())
                .eq(constants::COMMENT)
            {
                self.stream.advance_by(constants::COMMENT.len());

                let is_end_of_comment = self.stream.expect_and_skip_cond(b'>');

                if is_end_of_comment {
                    return Some(self.stream.slice_unchecked(start, self.stream.idx));
                }
            }

            self.stream.advance();
        }

        None
    }

    fn parse_attribute(&mut self) -> Option<(&'a [u8], Option<&'a [u8]>)> {
        let name = self.read_ident()?;
        self.skip_whitespaces();

        let has_value = self.stream.expect_and_skip_cond(b'=');
        if !has_value {
            return Some((name, None));
        }

        self.skip_whitespaces();
        let quote = self.stream.expect_oneof_and_skip(&[b'"', b'\''])?;

        let value = self.read_to(&[quote]);

        Some((name, Some(value)))
    }

    fn parse_attributes(&mut self) -> Attributes<'a> {
        let mut attributes = Attributes::new();

        while !self.stream.is_eof() {
            self.skip_whitespaces();

            let cur = self.stream.current_unchecked();

            if constants::SELF_CLOSING.contains(cur) {
                break;
            }

            if let Some((k, v)) = self.parse_attribute() {
                // `id` and `class` attributes need to be handled manually,
                // as we're going to store them in a HashMap so `get_element_by_id` is O(1)

                let v: Option<Bytes<'a>> = v.map(Into::into);

                if k.eq(constants::ID_ATTR) {
                    attributes.id = v.clone();
                } else if k.eq(constants::CLASS_ATTR) {
                    attributes.class = v.clone();
                }

                attributes.raw.insert(k.into(), v);
            }

            self.stream.advance();
        }

        attributes
    }

    fn parse_tag(&mut self, skip_current: bool) -> Option<Node<'a>> {
        let start = self.stream.idx;

        if skip_current {
            self.stream.next()?;
        }

        let markup_declaration = self.stream.expect_and_skip_cond(b'!');

        if markup_declaration {
            let is_comment = self
                .stream
                .slice(self.stream.idx, self.stream.idx + constants::COMMENT.len())
                .eq(constants::COMMENT);

            if is_comment {
                self.stream.advance_by(constants::COMMENT.len());
                let comment = self.skip_comment()?;

                // Comments are ignored, so we return no element
                return Some(Node::Comment(comment.into()));
            }

            let name = self.read_ident()?.to_ascii_uppercase();

            if name.eq(b"DOCTYPE") {
                self.skip_whitespaces();

                let is_html5 = self
                    .read_ident()
                    .map(|ident| ident.to_ascii_uppercase().eq(b"HTML"))
                    .unwrap_or(false);

                if is_html5 {
                    self.version = Some(HTMLVersion::HTML5);
                    self.skip_whitespaces();
                    self.stream.expect_and_skip(b'>')?;
                }

                // TODO: handle DOCTYPE for HTML version <5?

                return None;
            }

            // TODO: handle the case where <! is neither DOCTYPE nor a comment
            return None;
        }

        let name = self.read_ident()?;

        let attributes = self.parse_attributes();

        let mut children = InlineVec::new();

        let is_self_closing = self.stream.expect_and_skip_cond(b'/');

        self.skip_whitespaces();

        if is_self_closing {
            self.stream.expect_and_skip(b'>')?;

            let raw = self.stream.slice_from(start);

            // If this is a self-closing tag (e.g. <img />), we want to return early instead of
            // reading children as the next nodes don't belong to this tag
            return Some(Node::Tag(HTMLTag::new(
                Some(name.into()),
                attributes,
                children,
                raw.into(),
            )));
        }

        self.stream.expect_and_skip(b'>')?;

        if constants::VOID_TAGS.contains(&name) {
            let raw = self.stream.slice_from(start);

            // Some HTML tags don't have contents (e.g. <br>),
            // so we need to return early
            // Without it, any following tags would be sub-nodes
            return Some(Node::Tag(HTMLTag::new(
                Some(name.into()),
                attributes,
                children,
                raw.into(),
            )));
        }

        while !self.stream.is_eof() {
            self.skip_whitespaces();

            let idx = self.stream.idx;

            let slice = self.stream.slice(idx, idx + constants::END_OF_TAG.len());
            if slice.eq(constants::END_OF_TAG) {
                self.stream.advance_by(constants::END_OF_TAG.len());
                self.read_ident()?;

                // TODO: do we want to accept the tag if it has no closing tag?
                self.stream.expect_and_skip(b'>');
                break;
            }

            // TODO: "partial" JS parser is needed to deal with script tags
            let node_id = self.parse_single()?;

            children.push(node_id);
        }

        let raw = self.stream.slice_from(start);

        Some(Node::Tag(HTMLTag::new(
            Some(name.into()),
            attributes,
            children,
            raw.into(),
        )))
    }

    fn parse_single(&mut self) -> Option<NodeHandle> {
        self.skip_whitespaces();

        let ch = self.stream.current_cpy()?;

        if ch == constants::OPENING_TAG {
            if let Some(tag) = self.parse_tag(true) {
                let handle = self.register_tag(tag);
                let tag_id = handle.get_inner();

                let (id, class) = if let Some(Node::Tag(tag)) = self.tags.get(tag_id) {
                    (tag._attributes.id.clone(), tag._attributes.class.clone())
                } else {
                    (None, None)
                };

                if let Some(id) = id {
                    self.ids.insert(id.clone(), handle);
                }

                if let Some(class) = class {
                    self.process_class(&class, handle);
                }

                Some(handle)
            } else {
                None
            }
        } else {
            let node = Node::Raw(self.read_to(&[b'<']).into());
            let tag_id = self.register_tag(node);
            Some(tag_id)
        }
    }

    fn process_class(&mut self, class: &Bytes<'a>, element: NodeHandle) {
        let raw = class.raw();

        let mut stream = Stream::new(raw);

        let mut last = 0;

        while !stream.is_eof() {
            let cur = stream.current_unchecked();

            let is_last_char = stream.idx == raw.len() - 1;

            if util::is_strict_whitespace(*cur) || is_last_char {
                let idx = if is_last_char {
                    stream.idx + 1
                } else {
                    stream.idx
                };

                let slice = stream.slice(last, idx);
                if !slice.is_empty() {
                    self.classes
                        .entry(slice.into())
                        .or_insert_with(InlineVec::new)
                        .push(element);
                }

                last = idx + 1;
            }

            stream.advance();
        }
    }

    /// Resolves an internal Node ID obtained from a NodeHandle to a Node
    #[inline]
    pub fn resolve_node_id(&self, id: usize) -> Option<&Node<'a>> {
        self.tags.get(id)
    }

    pub(crate) fn parse(mut self) -> Parser<'a> {
        while !self.stream.is_eof() {
            if let Some(node) = self.parse_single() {
                self.ast.push(node);
            }
        }
        self
    }
}
