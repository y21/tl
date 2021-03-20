use crate::stream::Stream;
use crate::util;
use std::collections::HashMap;

const END_OF_TAG: &'static [u8] = &[b'<', b'/'];

#[derive(Debug)]
pub struct HTMLTag<'a> {
    _name: &'a [u8],
    _attributes: HashMap<&'a [u8], &'a [u8]>,
    _children: Vec<Node<'a>>
}

impl<'a> HTMLTag<'a> {
    pub fn new(
        name: &'a [u8],
        attr: HashMap<&'a [u8], &'a [u8]>,
        children: Vec<Node<'a>>
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children
        }
    }
}

#[derive(Debug)]
pub enum Node<'a> {
    Tag(HTMLTag<'a>),
    Raw(&'a [u8])
}

pub type Tree<'a> = Vec<Node<'a>>;

#[derive(Debug)]
pub struct Parser<'a> {
    stream: Stream<'a, u8>
}

impl<'a> Parser<'a> {
    pub fn new(input: &str) -> Parser {
        Parser {
            stream: Stream::new(input.as_bytes())
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
                break
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

    fn parse_attributes(&mut self) -> HashMap<&'a [u8], &'a [u8]> {
        let mut attr = HashMap::new();

        while !self.stream.is_eof() {
            let cur = self.stream.current_unchecked();
            // TODO: skip whitespaces?
            if *cur == b'>' {
                break
            }
            
            if let Some((k, v)) = self.parse_attribute() {
                attr.insert(k, v);
            }

            self.stream.idx += 1;
        }

        attr
    }

    fn parse_tag(&mut self, skip_current: bool) -> Option<HTMLTag<'a>> {
        if skip_current {
            self.stream.next()?;
        }

        let name = self.read_ident()?;
        let attr = self.parse_attributes(); // TODO: actually read attributes

        let mut children = Vec::new();
        
        // TODO: handle /> tags properly
        self.stream.expect_and_skip(b'/');
        self.stream.expect_and_skip(b'>')?;

        // TODO: parse children (read until we see > or />)
        while !self.stream.is_eof() {
            self.skip_whitespaces();

            let idx = self.stream.idx;

            let slice = self.stream.slice(idx, idx + END_OF_TAG.len());
            if slice.eq(END_OF_TAG) {
                self.stream.idx += END_OF_TAG.len();
                // TODO: make sure the identifier is the same as tag name!
                let ident = self.read_ident();

                self.stream.expect_and_skip(b'>')?;
                break
            }

            let node = self.parse_single()?;
            
            children.push(node);
        }

        let tag = HTMLTag::new(name, attr, children);
        

        Some(tag)
    }

    fn parse_single(&mut self) -> Option<Node<'a>> {
        self.skip_whitespaces();

        let ch = self.stream.current_unchecked_cpy();

        match ch {
            // TODO: if parse_tag fails (None case), we should probably just interpret it
            // as raw text...
            b'<' => self.parse_tag(true).and_then(|x| Some(Node::Tag(x))),
            _ => Some(Node::Raw(self.read_to(&[b'<'])))
        }
    }

    pub fn parse(&mut self) -> Tree<'a> {
        let mut tree = Vec::new();
        
        if let Some(node) = self.parse_single() {
            tree.push(node);
        }
        
        tree
    }
}