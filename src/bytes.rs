use core::{fmt, fmt::Debug};
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::ManuallyDrop,
};

/// A wrapper around raw bytes
#[derive(Eq, PartialOrd, Ord)]
pub struct Bytes<'a> {
    /// The inner data
    data: BytesInner,
    /// Enforce the lifetime of the referenced data
    _lt: PhantomData<&'a [u8]>,
}

/// The inner data of [`Bytes`]
///
/// Instead of using `&[u8]` and `Vec<u8>` for the variants,
/// we use raw pointers and a `u32` for the length.
/// This is to keep the size of the enum to 16 (on 64-bit machines),
/// which is the same as if this was just `struct Bytes<'a>(&'a [u8])`
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum BytesInner {
    /// Borrowed bytes
    Borrowed(*const u8, u32),
    /// Owned bytes
    Owned(*mut u8, u32),
}

impl<'a> PartialEq for Bytes<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let this = self.as_bytes();
        let that = other.as_bytes();
        this == that
    }
}

impl<'a> Hash for Bytes<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash must be implemented manually for Bytes, otherwise it would only hash a pointer
        let this = self.as_bytes();
        this.hash(state);
    }
}

impl<'a> Clone for Bytes<'a> {
    fn clone(&self) -> Self {
        // It is important to manually implement Clone for Bytes,
        // because if `self` was owned, then the default clone
        // implementation would only clone the pointer
        // which leads to aliasing boxes, and later, when `Bytes` is dropped,
        // the box is freed twice!
        match &self.data {
            BytesInner::Borrowed(data, len) => {
                Bytes::from(unsafe { compact_bytes_to_slice(*data, *len) })
            }
            BytesInner::Owned(data, len) => {
                let (ptr, len) = unsafe { clone_compact_bytes_parts(*data, *len) };
                Bytes {
                    data: BytesInner::Owned(ptr, len),
                    _lt: PhantomData,
                }
            }
        }
    }
}

impl<'a> From<&'a str> for Bytes<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        <Self as From<&'a [u8]>>::from(s.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(s: &'a [u8]) -> Self {
        Bytes {
            data: BytesInner::Borrowed(s.as_ptr(), s.len() as u32),
            _lt: PhantomData,
        }
    }
}

/// Converts `Bytes` raw parts to a slice
#[inline]
unsafe fn compact_bytes_to_slice<'a>(ptr: *const u8, l: u32) -> &'a [u8] {
    std::slice::from_raw_parts(ptr, l as usize)
}

/// Converts a boxed byte slice to compact raw parts
///
/// The caller is responsible for freeing the returned pointer and that the length of the slice does not overflow a u32!
unsafe fn boxed_slice_into_compact_parts(slice: Box<[u8]>) -> (*mut u8, u32) {
    // wrap box in `ManuallyDrop` so it's not dropped at the end of the scope
    let mut slice = ManuallyDrop::new(slice);
    let len = slice.len();
    let ptr = slice.as_mut_ptr();

    (ptr, len as u32)
}

/// Clones a slice given its raw parts and returns the new, cloned parts
#[inline]
unsafe fn clone_compact_bytes_parts(ptr: *mut u8, len: u32) -> (*mut u8, u32) {
    let slice = compact_bytes_to_slice(ptr, len).to_vec().into_boxed_slice();
    boxed_slice_into_compact_parts(slice)
}

// Custom `Debug` trait is implemented which displays the data as a UTF8 string,
// to make it easier to read for humans when logging
impl<'a> Debug for Bytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Bytes").field(&self.as_utf8_str()).finish()
    }
}

impl<'a> Bytes<'a> {
    /// Creates an empty `Bytes`
    #[inline]
    pub fn new() -> Self {
        Self {
            data: BytesInner::Borrowed("".as_bytes().as_ptr(), 0),
            _lt: PhantomData,
        }
    }

    /// Convenient method for lossy-encoding the data as UTF8
    #[inline]
    pub fn as_utf8_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    /// Tries to convert the inner data to a `&str`, without allocating in the case
    /// that the inner data is not valid UTF8
    #[inline]
    pub fn try_as_utf8_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }

    /// Returns the raw data wrapped by this struct
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match &self.data {
            BytesInner::Borrowed(b, l) => unsafe { compact_bytes_to_slice(*b, *l) },
            BytesInner::Owned(o, l) => unsafe { compact_bytes_to_slice(*o, *l) },
        }
    }

    /// Returns the raw data referenced by this struct
    ///
    /// The lifetime of the returned data is tied to 'a, unlike `Bytes::as_bytes`
    /// which has a lifetime of '_ (self) in case it is owned
    #[inline]
    pub fn as_bytes_borrowed(&self) -> Option<&'a [u8]> {
        match &self.data {
            BytesInner::Borrowed(b, l) => Some(unsafe { compact_bytes_to_slice(*b, *l) }),
            _ => None,
        }
    }

    /// Returns a read-only raw pointer to the inner data
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        match &self.data {
            BytesInner::Borrowed(b, _) => *b,
            BytesInner::Owned(o, _) => *o,
        }
    }

    /// Sets the inner data to the given data and returns the old bytes
    pub fn set<B: IntoOwnedBytes>(&mut self, data: B) -> Result<Option<Box<[u8]>>, SetBytesError> {
        const MAX: usize = u32::MAX as usize;

        let data = <B as IntoOwnedBytes>::into_bytes(data);

        if data.len() > MAX {
            return Err(SetBytesError::LengthOverflow);
        }

        // SAFETY: All invariants are checked
        Ok(unsafe { self.set_unchecked(data) })
    }

    /// Sets the inner data to the given data without checking for validity of the data
    ///
    /// ## Safety
    /// - Once `data` is converted to a `Box<[u8]>`, its length must not be greater than u32::MAX
    #[inline]
    pub unsafe fn set_unchecked<B: IntoOwnedBytes>(&mut self, data: B) -> Option<Box<[u8]>> {
        let data = <B as IntoOwnedBytes>::into_bytes(data);

        let (ptr, len) = boxed_slice_into_compact_parts(data);

        let bytes = BytesInner::Owned(ptr, len);
        let old = std::mem::replace(&mut self.data, bytes);

        // we cannot let Drop code run because that would deallocate `old`
        let old = ManuallyDrop::new(old);

        match &*old {
            BytesInner::Borrowed(_, _) => None,
            BytesInner::Owned(ptr, len) => {
                let len = *len as usize;
                Some(Vec::from_raw_parts(*ptr, len, len).into_boxed_slice())
            }
        }
    }
}

mod private {
    pub trait Sealed {}
}

/// A trait implemented on types that can be used for `Bytes::set`.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait IntoOwnedBytes: private::Sealed {
    fn into_bytes(self) -> Box<[u8]>;
}

macro_rules! impl_into_owned_bytes_trivial {
    ($($t:ty),*) => {
        $(
            impl private::Sealed for $t {}
            impl IntoOwnedBytes for $t {
                #[inline]
                fn into_bytes(self) -> Box<[u8]> {
                    self.into()
                }
            }
        )*
    };
}

impl_into_owned_bytes_trivial!(Box<[u8]>, &[u8], Vec<u8>);

impl private::Sealed for &str {}
impl IntoOwnedBytes for &str {
    #[inline]
    fn into_bytes(self) -> Box<[u8]> {
        self.as_bytes().into()
    }
}

impl private::Sealed for String {}
impl IntoOwnedBytes for String {
    #[inline]
    fn into_bytes(self) -> Box<[u8]> {
        self.into_bytes().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SetBytesError {
    /// The length of the given data would overflow a `u32`
    LengthOverflow,
}

impl Drop for BytesInner {
    fn drop(&mut self) {
        // we only need to deallocate if we own the data
        // if we don't, just do nothing
        if let BytesInner::Owned(ptr, len) = self {
            let ptr = *ptr;
            let len = *len as usize;

            // carefully reconstruct a `Box<[u8]>` from the raw pointer and length
            // and immediately drop it to free memory
            unsafe { drop(Vec::from_raw_parts(ptr, len, len).into_boxed_slice()) };
        }
    }
}
