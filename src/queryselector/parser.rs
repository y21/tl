use crate::{stream::Stream, util};

use super::Selector;

/// A query selector parser
pub struct Parser<'a> {
    stream: Stream<'a, u8>,
}

impl<'a> Parser<'a> {
    /// Creates a new query selector parser
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            stream: Stream::new(input),
        }
    }

    fn skip_whitespaces(&mut self) -> bool {
        let has_whitespace = self.stream.expect_and_skip_cond(b' ');
        while !self.stream.is_eof() {
            if self.stream.expect_and_skip(b' ').is_none() {
                break;
            }
        }
        has_whitespace
    }

    fn read_identifier(&mut self) -> &'a [u8] {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let is_ident = self.stream.current().copied().map_or(false, util::is_ident);
            if !is_ident {
                break;
            } else {
                self.stream.advance();
            }
        }

        self.stream.slice(start, self.stream.idx)
    }

    fn parse_attribute(&mut self) -> Option<Selector<'a>> {
        let attribute = self.read_identifier();
        let ty = match self.stream.current_cpy() {
            Some(b']') => {
                self.stream.advance();
                Selector::Attribute(attribute)
            }
            Some(b'=') => {
                self.stream.advance();
                let quote = self.stream.expect_oneof_and_skip(&[b'"', b'\'']);
                let value = self.read_identifier();
                if let Some(quote) = quote {
                    // Only require the given quote if the value starts with a quote
                    self.stream.expect_and_skip(quote)?;
                }
                self.stream.expect_and_skip(b']')?;
                Selector::AttributeValue(attribute, value)
            }
            Some(c @ b'~' | c @ b'^' | c @ b'$' | c @ b'*') => {
                self.stream.advance();
                self.stream.expect_and_skip(b'=')?;
                let quote = self.stream.expect_oneof_and_skip(&[b'"', b'\'']);
                let value = self.read_identifier();
                if let Some(quote) = quote {
                    // Only require the given quote if the value starts with a quote
                    self.stream.expect_and_skip(quote)?;
                }
                self.stream.expect_and_skip(b']')?;
                match c {
                    b'~' => Selector::AttributeValueWhitespacedContains(attribute, value),
                    b'^' => Selector::AttributeValueStartsWith(attribute, value),
                    b'$' => Selector::AttributeValueEndsWith(attribute, value),
                    b'*' => Selector::AttributeValueSubstring(attribute, value),
                    _ => unreachable!(),
                }
            }
            _ => return None,
        };
        Some(ty)
    }

    /// Parses a selectors group production as defined in the [grammar provided by the W3C].
    ///
    /// [grammar provided by the W3C]: https://www.w3.org/TR/selectors-3/#w3cselgrammar
    pub fn parse_selectors_group(&mut self) -> Option<Selector<'a>> {
        let mut left = self.parse_selector()?;

        while let Some(right) = self.parse_selector() {
            left = Selector::Or(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// Parses a selector production as defined in the [grammar provided by the W3C].
    ///
    /// [grammar provided by the W3C]: https://www.w3.org/TR/selectors-3/#w3cselgrammar
    fn parse_selector(&mut self) -> Option<Selector<'a>> {
        let mut left = self.parse_simple_selector_sequence()?;
        let mut has_whitespaces = self.skip_whitespaces();

        while let Some(tok) = self.stream.current_cpy() {
            match tok {
                b',' => {
                    self.stream.advance();
                    return Some(left);
                }
                b'>' => {
                    self.stream.advance();
                    let right = self.parse_simple_selector_sequence()?;
                    left = Selector::Parent(Box::new(left), Box::new(right));
                }
                _ if has_whitespaces => {
                    let right = self.parse_simple_selector_sequence()?;
                    left = Selector::Descendant(Box::new(left), Box::new(right));
                }
                _ => unreachable!(),
            }

            has_whitespaces = self.skip_whitespaces();
        }

        Some(left)
    }

    /// Parses a simple selector sequence production as defined in the [grammar provided by the W3C].
    ///
    /// [grammar provided by the W3C]: https://www.w3.org/TR/selectors-3/#w3cselgrammar
    fn parse_simple_selector_sequence(&mut self) -> Option<Selector<'a>> {
        let mut result = None;
        self.skip_whitespaces();

        while let Some(right) = match self.stream.current_cpy() {
            Some(b'#') => {
                self.stream.advance();
                let id = self.read_identifier();
                Some(Selector::Id(id))
            }
            Some(b'.') => {
                self.stream.advance();
                let class = self.read_identifier();
                Some(Selector::Class(class))
            }
            Some(b'*') => {
                self.stream.advance();
                Some(Selector::All)
            }
            Some(b'[') => {
                self.stream.advance();
                self.parse_attribute()
            }
            Some(tok) if util::is_ident(tok) => {
                let tag = self.read_identifier();
                Some(Selector::Tag(tag))
            }
            _ => None,
        } {
            if let Some(left) = result {
                result = Some(Selector::And(Box::new(left), Box::new(right)));
            } else {
                result = Some(right);
            }
        }

        result
    }
}
