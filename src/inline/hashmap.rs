use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::{collections::HashMap, mem::MaybeUninit};

/// Similar to InlineVec, this structure will use an array
/// if it is small enough to live on the stack, otherwise
/// it allocates a HashMap on the heap
///
/// Hashing can be slower than just iterating through an array
/// if the array is small, which is where it makes most sense
// #[derive(Debug, Clone)]
///
/// To convert to a HashMap, use `HashMap::from()`
pub enum InlineHashMap<K, V, const N: usize> {
    /// Inline array
    Inline {
        /// Length of the array
        len: usize,
        /// Data stored in the array
        data: [MaybeUninit<(K, V)>; N],
    },
    /// Heap allocated HashMap
    Heap(HashMap<K, V>),
}

impl<K, V, const N: usize> Debug for InlineHashMap<K, V, N>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InlineHashMap<{} items>", self.len())
    }
}

impl<K, V, const N: usize> Clone for InlineHashMap<K, V, N>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Heap(m) => Self::Heap(m.clone()),
            Self::Inline { len, data } => {
                let mut new_data = super::uninit_array();

                let iter = data.iter().take(*len).enumerate();

                for (idx, element) in iter {
                    let element = unsafe { &*element.as_ptr() };
                    let (key, value) = element.clone();
                    new_data[idx] = MaybeUninit::new((key, value));
                }

                Self::Inline {
                    len: *len,
                    data: new_data,
                }
            }
        }
    }
}

impl<K, V, const N: usize> From<InlineHashMap<K, V, N>> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(map: InlineHashMap<K, V, N>) -> Self {
        match map {
            InlineHashMap::Heap(m) => m,
            InlineHashMap::Inline { len, data } => {
                let mut new_data = HashMap::with_capacity(len);

                let iter = data.into_iter().take(len);

                for element in iter {
                    let (k, v) = unsafe { element.assume_init() };
                    new_data.insert(k, v);
                }

                new_data
            }
        }
    }
}

impl<K, V, const N: usize> InlineHashMap<K, V, N> {
    /// Creates a new InlineHashMap
    #[inline]
    pub fn new() -> Self {
        Self::Inline {
            len: 0,
            data: super::uninit_array(),
        }
    }

    /// Returns the length of this map
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Heap(map) => map.len(),
        }
    }

    /// Checks whether this vector is allocated on the heap
    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self, Self::Heap(_))
    }
}

impl<K: Eq + Hash, V, const N: usize> InlineHashMap<K, V, N> {
    /// Returns a reference to the value corresponding to the key.
    pub fn get<'m>(&'m self, k: &K) -> Option<&'m V> {
        match self {
            Self::Inline { data, len } => unsafe {
                InlineHashMapIterator::new(data, *len)
                    .find(|(key, _)| key.eq(k))
                    .map(|(_, value)| value)
            },
            Self::Heap(map) => map.get(k),
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, k: K, v: V) {
        let (array, len) = match self {
            Self::Inline { data, len } => (data, len),
            Self::Heap(map) => {
                map.insert(k, v);
                return;
            }
        };

        if *len >= N {
            let mut map = HashMap::with_capacity(*len);

            // move old elements to heap
            for element in array.iter_mut().take(*len) {
                let element = std::mem::replace(element, MaybeUninit::uninit());
                let (key, value) = unsafe { element.assume_init() };

                map.insert(key, value);
            }

            // insert new element
            map.insert(k, v);
            *self = Self::Heap(map);
            return;
        } else {
            array[*len].write((k, v));
            *len += 1;
        }
    }

    /// Returns true if the map contains a value for the specified key.
    pub fn contains_key(&self, k: &K) -> bool {
        match self {
            Self::Inline { data, len } => unsafe {
                InlineHashMapIterator::new(data, *len).any(|(key, _)| key.eq(k))
            },
            Self::Heap(map) => map.contains_key(k),
        }
    }
}

/// An iterator over the inline array elements of an `InlineHashMap`.
pub struct InlineHashMapIterator<'a, K, V> {
    array: &'a [MaybeUninit<(K, V)>],
    idx: usize,
    len: usize,
}

impl<'a, K, V> InlineHashMapIterator<'a, K, V> {
    pub(crate) unsafe fn new(array: &'a [MaybeUninit<(K, V)>], len: usize) -> Self {
        Self { array, idx: 0, len }
    }
}

impl<'a, K, V> Iterator for InlineHashMapIterator<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }

        let element = unsafe { &*self.array[self.idx].as_ptr() };
        self.idx += 1;

        Some(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlinehashmap_clone() {
        let mut x = InlineHashMap::<usize, usize, 4>::new();

        for i in 0..10 {
            x.insert(i, i * 2);
        }

        let x = x.clone();
        assert_eq!(x.len(), 10);
        assert!(x.is_heap_allocated());
        assert_eq!(x.get(&9), Some(&18));
    }

    #[test]
    fn inlinehashmap() {
        let mut x = InlineHashMap::<&'static str, usize, 4>::new();
        assert_eq!(x.len(), 0);
        assert_eq!(x.get(&"hi"), None);
        assert!(!x.is_heap_allocated());

        x.insert("foo", 1337);
        assert_eq!(x.len(), 1);
        assert_eq!(x.get(&"foo"), Some(&1337));
        assert!(!x.is_heap_allocated());

        x.insert("foo2", 2);
        x.insert("foo3", 3);
        x.insert("foo4", 4);

        assert_eq!(x.len(), 4);

        x.insert("foo5", 5);
        assert_eq!(x.len(), 5);
        assert!(x.is_heap_allocated());

        x.insert("foo6", 6);
        x.insert("foo7", 7);
        x.insert("foo8", 8);
        x.insert("foo9", 9);
        x.insert("foo10", 10);
        x.insert("foo11", 11);
    }
}
