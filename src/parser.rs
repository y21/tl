use crate::stream::Stream;
use crate::util;

pub struct HTMLTag<'a> {
    _name: &'a u8
}

pub enum Node<'a> {
    Tag(HTMLTag<'a>),
    Raw(&'a u8)
}

pub type Tree<'a> = Vec<Node<'a>>;

pub struct Parser<'a> {
    stream: Stream<'a, u8>
}

impl<'a> Parser<'a> {
    pub fn new(input: &str) -> Parser {
        Parser {
            stream: Stream::new(input.as_bytes())
        }
    }

    pub fn skip_whitespaces(&mut self) {
        self.read_to(&[b' ', b'\n']);
    }

    pub fn read_to(&mut self, terminator: &[u8]) {
        while !self.stream.is_eof() {
            let ch = self.stream.current_unchecked();

            if terminator.contains(ch) {
                break
            }
        }
    }

    fn read_ident(&mut self) -> Option<&[u8]> {
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

    fn read_tag(&mut self) -> Option<HTMLTag<'a>> {
        todo!()
    }

    fn parse_single(&mut self) -> Option<Node<'a>> {
        todo!()
    }

    pub fn parse(&mut self) -> Tree<'a> {
        todo!()
    }
}