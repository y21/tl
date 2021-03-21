use crate::bytes::BorrowedBytes;
use crate::stream::Stream;
use crate::util;
use std::collections::HashMap;

macro_rules! str_to_u8_arr {
    ($($st:expr),*) => {
        &[$($st.as_bytes()),*]
    }
}

const END_OF_TAG: &[u8] = &[b'<', b'/']; // </p>
const SELF_CLOSING: &[u8] = &[b'/', b'>']; // <br />
const COMMENT: &[u8] = &[b'-', b'-']; // <!-- -->
const VOID_TAGS: &[&[u8]] = str_to_u8_arr![
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta", "param",
    "source", "track", "wbr"
];

mod flags {
    pub const COMMENT: u32 = 1 << 0;
}

#[derive(Debug)]
pub struct HTMLTag<'a> {
    _name: Option<BorrowedBytes<'a>>,
    _attributes: HashMap<BorrowedBytes<'a>, BorrowedBytes<'a>>,
    _flags: u32,
    _children: Vec<Node<'a>>,
}

impl<'a> HTMLTag<'a> {
    pub fn new(
        name: Option<BorrowedBytes<'a>>,
        attr: HashMap<BorrowedBytes<'a>, BorrowedBytes<'a>>,
        children: Vec<Node<'a>>,
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children,
            _flags: 0,
        }
    }

    pub fn comment(mut self) -> Self {
        self._flags |= flags::COMMENT;
        self
    }
}

#[derive(Debug)]
pub enum Node<'a> {
    Tag(HTMLTag<'a>),
    Raw(BorrowedBytes<'a>),
}

pub type Tree<'a> = Vec<Node<'a>>;

#[derive(Debug)]
pub struct Parser<'a> {
    stream: Stream<'a, u8>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &str) -> Parser {
        Parser {
            stream: Stream::new(input.as_bytes()),
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

    fn parse_attribute(&mut self) -> Option<(&'a [u8], &'a [u8])> {
        let name = self.read_ident()?;
        self.skip_whitespaces();

        // TODO: allow attributes with no value?
        self.stream.expect_and_skip(b'=')?;

        self.skip_whitespaces();
        let quote = self.stream.expect_oneof_and_skip(&[b'"', b'\''])?;

        let value = self.read_to(&[quote]);

        Some((name, value))
    }

    fn parse_attributes(&mut self) -> HashMap<BorrowedBytes<'a>, BorrowedBytes<'a>> {
        let mut attr = HashMap::new();

        while !self.stream.is_eof() {
            self.skip_whitespaces();

            let cur = self.stream.current_unchecked();

            if SELF_CLOSING.contains(cur) {
                break;
            }

            if let Some((k, v)) = self.parse_attribute() {
                attr.insert(k.into(), v.into());
            }

            self.stream.idx += 1;
        }

        attr
    }

    fn parse_tag(&mut self, skip_current: bool) -> Option<HTMLTag<'a>> {
        if skip_current {
            self.stream.next()?;
        }

        let markup_declaration = self.stream.expect_and_skip_cond(b'!');

        if markup_declaration {
            let is_comment = self
                .stream
                .slice(self.stream.idx, self.stream.idx + COMMENT.len())
                .eq(COMMENT);

            if is_comment {
                self.stream.idx += COMMENT.len();
                self.skip_comment();

                // Comments are ignored, so we return no element
                return Some(HTMLTag::new(None, HashMap::new(), Vec::new()).comment());
            }

            let name = self.read_ident()?.to_ascii_uppercase();

            if name.eq("DOCTYPE".as_bytes()) {
                // TODO: handle doctype
                todo!();
            }

            // TODO: handle the case where <! is neither DOCTYPE nor a comment
            todo!();
        }

        let name = self.read_ident()?;

        let attr = self.parse_attributes();

        let mut children = Vec::new();

        let is_self_closing = self.stream.expect_and_skip_cond(b'/');

        self.skip_whitespaces();

        if is_self_closing {
            self.stream.expect_and_skip(b'>')?;

            // If this is a self-closing tag (e.g. <img />), we want to return early instead of
            // reading children as the next nodes don't belong to this tag
            return Some(HTMLTag::new(Some(name.into()), attr, children));
        }

        self.stream.expect_and_skip(b'>')?;

        if VOID_TAGS.contains(&name) {
            // Some HTML tags don't have contents (e.g. <br>),
            // so we need to return early
            // Without it, any following tags would be sub-nodes
            return Some(HTMLTag::new(Some(name.into()), attr, children));
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

            children.push(node);
        }

        let tag = HTMLTag::new(Some(name.into()), attr, children);

        Some(tag)
    }

    fn parse_single(&mut self) -> Option<Node<'a>> {
        self.skip_whitespaces();

        let ch = self.stream.current_cpy()?;

        match ch {
            // TODO: if parse_tag fails (None case), we should probably just interpret it
            // as raw text...
            b'<' => self.parse_tag(true).map(Node::Tag),
            _ => Some(Node::Raw(self.read_to(&[b'<']).into())),
        }
    }

    pub fn as_ast(&mut self) -> Tree<'a> {
        let mut tree = Vec::new();

        while !self.stream.is_eof() {
            if let Some(node) = self.parse_single() {
                tree.push(node);
            }
        }

        tree
    }
}
