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

/// An error that occurred during a call to `Bytes::set`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SetBytesError {
    /// The length of the given data would overflow a `u32`
    LengthOverflow,
}

impl fmt::Display for SetBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            SetBytesError::LengthOverflow => {
                write!(f, "The string length is too large to fit in a `u32`")
            }
        }
    }
}

impl Error for SetBytesError {}
