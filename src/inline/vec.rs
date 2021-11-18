use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;
use std::ops::Index;

/// A wrapper around a `Vec<T>` that lives on the stack
/// if it is small enough
///
/// To convert to a `Vec<T>` use `Vec::from`
pub enum InlineVec<T, const N: usize> {
    /// Inline array
    Inline {
        /// Length of the array
        len: usize,
        /// Data stored in the array
        data: [MaybeUninit<T>; N],
    },
    /// Heap allocated Vec
    Heap(Vec<T>),
}

impl<T, const N: usize> Debug for InlineVec<T, N>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InlineVec<{} items>", self.len())
    }
}

impl<T, const N: usize> Clone for InlineVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Heap(m) => Self::Heap(m.clone()),
            Self::Inline { len, data } => {
                let mut new_data = super::uninit_array();

                let iter = data.iter().take(*len).enumerate();

                for (idx, element) in iter {
                    let element = unsafe { &*element.as_ptr() };
                    new_data[idx] = MaybeUninit::new(T::clone(element));
                }

                Self::Inline {
                    len: *len,
                    data: new_data,
                }
            }
        }
    }
}

impl<T, const N: usize> From<InlineVec<T, N>> for Vec<T> {
    fn from(vec: InlineVec<T, N>) -> Self {
        match vec {
            InlineVec::Heap(m) => m,
            InlineVec::Inline { len, data } => {
                let mut new_data = Vec::with_capacity(len);

                let iter = data.into_iter().take(len);

                for element in iter {
                    new_data.push(unsafe { element.assume_init() });
                }

                new_data
            }
        }
    }
}

impl<T, const N: usize> InlineVec<T, N> {
    /// Creates a new InlineVec
    #[inline]
    pub fn new() -> Self {
        Self::Inline {
            len: 0,
            data: super::uninit_array(),
        }
    }

    /// Returns a slice to the contents of this vector
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Heap(v) => v.as_slice(),
            Self::Inline { len, data } => unsafe {
                std::slice::from_raw_parts(data.as_ptr() as *const T, *len)
            },
        }
    }

    /// Returns an iterator over the elements of the vector
    pub fn iter(&self) -> InlineVecIter<'_, T, N> {
        InlineVecIter { idx: 0, vec: self }
    }

    /// Returns the length of this vector
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Heap(vec) => vec.len(),
        }
    }

    /// Returns the element at a given index
    pub fn get(&self, idx: usize) -> Option<&T> {
        match self {
            Self::Inline { data, len } => {
                if idx < *len {
                    Some(unsafe { &*data.get_unchecked(idx).as_ptr() })
                } else {
                    None
                }
            }
            Self::Heap(vec) => vec.get(idx),
        }
    }

    /// Pushes a new element to the end of this vector
    pub fn push(&mut self, value: T) -> usize {
        let (array, len) = match self {
            Self::Inline { data, len } => (data, len),
            Self::Heap(vec) => {
                vec.push(value);
                return vec.len() - 1;
            }
        };

        if *len >= N {
            let mut vec = Vec::with_capacity(*len);

            // move old elements to heap
            for element in array.iter_mut().take(*len) {
                let element = std::mem::replace(element, MaybeUninit::uninit());

                vec.push(unsafe { element.assume_init() });
            }

            // push the new element
            vec.push(value);
            let len = vec.len() - 1;

            *self = InlineVec::Heap(vec);

            len - 1
        } else {
            array[*len].write(value);
            *len += 1;
            *len - 1
        }
    }

    /// Checks whether this vector is allocated on the heap
    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self, Self::Heap(_))
    }
}

impl<T, const N: usize> Index<usize> for InlineVec<T, N> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx).expect("index out of bounds")
    }
}

/// An iterator over the elements stored in an [`InlineVec`]
pub struct InlineVecIter<'a, T, const N: usize> {
    vec: &'a InlineVec<T, N>,
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for InlineVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        self.vec.get(self.idx - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlinevec_iter() {
        let mut x = InlineVec::<usize, 2>::new();
        x.push(13);
        x.push(42);
        x.push(17);
        x.push(19);
        let mut iter = x.iter();
        assert_eq!(iter.next(), Some(&13));
        assert_eq!(iter.next(), Some(&42));
        assert_eq!(iter.next(), Some(&17));
        assert_eq!(iter.next(), Some(&19));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn inlinevec() {
        let mut x = InlineVec::<usize, 4>::new();
        assert_eq!(x.len(), 0);
        assert_eq!(x.get(0), None);
        assert!(!x.is_heap_allocated());

        x.push(1337);
        assert_eq!(x.len(), 1);
        assert_eq!(x.get(0), Some(&1337));
        assert!(!x.is_heap_allocated());

        for v in 0..3 {
            x.push(v);
        }

        assert_eq!(x.len(), 4);

        // this call should move the vector to the heap
        x.push(42);
        assert_eq!(x.len(), 5);
        assert!(x.is_heap_allocated());

        // check that the old vector is still valid
        assert_eq!(x.get(0), Some(&1337));

        for v in 0..500 {
            x.push(v);
        }

        assert_eq!(x.len(), 505);
        assert!(x.is_heap_allocated());

        assert_eq!(x.get(1337), None);
    }
}
