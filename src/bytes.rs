use core::{fmt, fmt::Debug};
use std::borrow::Cow;

/// A wrapper around a DST-slice
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Bytes<'a>(&'a [u8]);

impl<'a> From<&'a str> for Bytes<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        Self(s)
    }
}

/// Custom `Debug` trait is implemented which displays the data as a UTF8 string,
/// to make it easier to read for humans when logging
impl<'a> Debug for Bytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Bytes").field(&self.as_utf8_str()).finish()
    }
}

impl<'a> Bytes<'a> {
    /// Convenient method for lossy-encoding the data as UTF8
    pub fn as_utf8_str(&self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.0)
    }

    /// Returns the raw data wrapped by this struct
    pub fn raw(&self) -> &'a [u8] {
        self.0
    }

    /// Returns a read-only raw pointer to the inner data
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}
