use core::fmt;
use std::error::Error;

/// An error that occurred during parsing
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    /// The input string length was too large to fit in a `u32`
    InvalidLength,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ParseError::InvalidLength => {
                write!(f, "The input string length is too large to fit in a `u32`")
            }
        }
    }
}

impl Error for ParseError {}
