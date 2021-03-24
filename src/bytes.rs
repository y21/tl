use core::{fmt, fmt::Debug};
use std::borrow::Cow;


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowedBytes<'a>(pub &'a [u8]);

impl<'a> From<&'a str> for BorrowedBytes<'a> {
    fn from(s: &'a str) -> Self {
        Self(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for BorrowedBytes<'a> {
    fn from(s: &'a [u8]) -> Self {
        Self(s)
    }
}

impl<'a> Debug for BorrowedBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BorrowedBytes")
            .field(&self.as_utf8_str())
            .finish()
    }
}

impl<'a> BorrowedBytes<'a> {
    pub fn as_utf8_str(&self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.0)
    }
}

pub trait AsBytes {
    fn as_bytes<'a>(&'a self) -> BorrowedBytes<'a>;
}

impl AsBytes for String {
    fn as_bytes(&self) -> BorrowedBytes<'_> {
        BorrowedBytes::from(&self[..])
    }
}

macro_rules! asbytes_from_impl {
    ($t:ty) => {
        impl AsBytes for $t {
            fn as_bytes(&self) -> BorrowedBytes<'_> {
                BorrowedBytes::from(self)
            }
        }
    }
}

asbytes_from_impl!(str);
asbytes_from_impl!([u8]);