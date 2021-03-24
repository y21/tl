use crate::bytes::BorrowedBytes;
use crate::stream::Stream;
use crate::util;
use std::{collections::HashMap, rc::Rc};

macro_rules! str_to_u8_arr {
    ($($st:expr),*) => {
        &[$($st.as_bytes()),*]
    }
}

const OPENING_TAG: u8 = b'<';
const END_OF_TAG: &[u8] = &[b'<', b'/']; // </p>
const SELF_CLOSING: &[u8] = &[b'/', b'>']; // <br />
const COMMENT: &[u8] = &[b'-', b'-']; // <!-- -->
const ID_ATTR: &[u8] = "id".as_bytes();
const CLASS_ATTR: &[u8] = "class".as_bytes();
const VOID_TAGS: &[&[u8]] = str_to_u8_arr![
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta", "param",
    "source", "track", "wbr"
];

mod flags {
    pub const COMMENT: u32 = 1 << 0;
}

#[derive(Debug, Clone)]
pub struct Attributes<'a> {
    pub raw: HashMap<BorrowedBytes<'a>, Option<BorrowedBytes<'a>>>,
    pub id: Option<BorrowedBytes<'a>>,
    pub class: Option<BorrowedBytes<'a>>,
}

impl<'a> Attributes<'a> {
    pub fn new() -> Self {
        Self {
            raw: HashMap::new(),
            id: None,
            class: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HTMLTag<'a> {
    _name: Option<BorrowedBytes<'a>>,
    _attributes: Attributes<'a>,
    _flags: u32,
    _children: Vec<Rc<Node<'a>>>,
    _start: usize,
    _end: usize
}

impl<'a> HTMLTag<'a> {
    pub fn new(
        name: Option<BorrowedBytes<'a>>,
        attr: Attributes<'a>,
        children: Vec<Rc<Node<'a>>>,
        start: usize,
        end: usize
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children,
            _flags: 0,
            _start: start,
            _end: end
        }
    }

    pub(crate) fn add_child(&mut self, c: Rc<Node<'a>>) {
        self._children.push(c);
    }

    pub(crate) fn comment(mut self) -> Self {
        self._flags |= flags::COMMENT;
        self
    }
}

#[derive(Debug, Clone)]
pub enum Node<'a> {
    Tag(HTMLTag<'a>),
    Raw(BorrowedBytes<'a>),
    Comment(BorrowedBytes<'a>)
}

pub type Tree<'a> = Vec<Rc<Node<'a>>>;

#[derive(Debug)]
pub struct Parser<'a> {
    pub stream: Stream<'a, u8>,
    pub ast: Tree<'a>,
    pub ids: HashMap<BorrowedBytes<'a>, Rc<Node<'a>>>,
    pub classes: HashMap<BorrowedBytes<'a>, Rc<Node<'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &str) -> Parser {
        Parser {
            stream: Stream::new(input.as_bytes()),
            ast: Vec::new(),
            ids: HashMap::new(),
            classes: HashMap::new(),
        }
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

            self.stream.idx += 1;
        }

        self.stream.slice_unchecked(start, self.stream.idx)
    }

    fn read_while(&mut self, terminator: &[u8]) {
        while !self.stream.is_eof() {
            let ch = self.stream.current_unchecked();

            if !terminator.contains(ch) {
                break;
            }

            self.stream.idx += 1;
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

            self.stream.idx += 1;
        }

        None
    }

    fn skip_comment(&mut self) -> Option<&'a [u8]> {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let idx = self.stream.idx;

            if self.stream.slice_len(idx, COMMENT.len()).eq(COMMENT) {
                self.stream.idx += COMMENT.len();

                let is_end_of_comment = self.stream.expect_and_skip_cond(b'>');

                if is_end_of_comment {
                    return Some(self.stream.slice_unchecked(start, self.stream.idx));
                }
            }

            self.stream.idx += 1;
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

            if SELF_CLOSING.contains(cur) {
                break;
            }

            if let Some((k, v)) = self.parse_attribute() {
                // `id` and `class` attributes need to be handled manually,
                // as we're going to store them in a HashMap so `get_element_by_id` is O(1)

                let v: Option<BorrowedBytes<'a>> = v.map(Into::into);

                if k.eq(ID_ATTR) {
                    attributes.id = v.clone();
                } else if k.eq(CLASS_ATTR) {
                    // TODO: This isn't correct - `class` attribute is space delimited
                    attributes.class = v.clone();
                }

                attributes.raw.insert(k.into(), v);
            }

            self.stream.idx += 1;
        }

        attributes
    }

    fn parse_tag(&mut self, skip_current: bool) -> Option<Node<'a>> {
        if skip_current {
            self.stream.next()?;
        }

        let start = self.stream.idx;

        let markup_declaration = self.stream.expect_and_skip_cond(b'!');

        if markup_declaration {
            let is_comment = self
                .stream
                .slice(self.stream.idx, self.stream.idx + COMMENT.len())
                .eq(COMMENT);

            if is_comment {
                self.stream.idx += COMMENT.len();
                let comment = self.skip_comment()?;

                // Comments are ignored, so we return no element
                return Some(Node::Comment(comment.into()));
            }

            let name = self.read_ident()?.to_ascii_uppercase();

            if name.eq("DOCTYPE".as_bytes()) {
                // TODO: handle doctype
                todo!();
            }

            // TODO: handle the case where <! is neither DOCTYPE nor a comment
            return None;
        }

        let name = self.read_ident()?;

        let attributes = self.parse_attributes();

        let mut element = HTMLTag::new(Some(name.into()), attributes, Vec::new(), start, 0);

        let is_self_closing = self.stream.expect_and_skip_cond(b'/');

        self.skip_whitespaces();

        if is_self_closing {
            self.stream.expect_and_skip(b'>')?;

            element._end = self.stream.idx;

            // If this is a self-closing tag (e.g. <img />), we want to return early instead of
            // reading children as the next nodes don't belong to this tag
            return Some(Node::Tag(element));
        }

        self.stream.expect_and_skip(b'>')?;

        if VOID_TAGS.contains(&name) {
            element._end = self.stream.idx;

            // Some HTML tags don't have contents (e.g. <br>),
            // so we need to return early
            // Without it, any following tags would be sub-nodes
            return Some(Node::Tag(element));
        }

        while !self.stream.is_eof() {
            self.skip_whitespaces();

            let idx = self.stream.idx;

            let slice = self.stream.slice(idx, idx + END_OF_TAG.len());
            if slice.eq(END_OF_TAG) {
                self.stream.idx += END_OF_TAG.len();
                let ident = self.read_ident()?;

                if !ident.eq(name) {
                    return None;
                }

                // TODO: do we want to accept the tag if it has no closing tag?
                break;
            }

            // TODO: "partial" JS parser is needed to deal with script tags
            let node = self.parse_single()?;

            element.add_child(node);
        }

        element._end = self.stream.idx;
        Some(Node::Tag(element))
    }

    fn parse_single(&mut self) -> Option<Rc<Node<'a>>> {
        self.skip_whitespaces();

        let ch = self.stream.current_unchecked_cpy();

        if ch == OPENING_TAG {
            if let Some(tag) = self.parse_tag(true) {
                let tag_rc = Rc::new(tag);

                if let Node::Tag(tag) = &*tag_rc {
                    let (id, class) = (&tag._attributes.id, &tag._attributes.class);

                    if let Some(id) = id {
                        self.ids.insert(id.clone(), tag_rc.clone());
                    }

                    if let Some(class) = class {
                        self.classes.insert(class.clone(), tag_rc.clone());
                    }
                }

                Some(tag_rc)
            } else {
                None
            }
        } else {
            Some(Rc::new(Node::Raw(self.read_to(&[b'<']).into())))
        }
    }

    pub fn parse(mut self) -> Parser<'a> {
        while !self.stream.is_eof() {
            if let Some(node) = self.parse_single() {
                self.ast.push(node);
            }
        }
        self
    }
}
