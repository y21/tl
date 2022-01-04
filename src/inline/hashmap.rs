use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ptr;
use std::{collections::HashMap, mem::MaybeUninit};

/// Similar to InlineVec, this structure will use an array
/// if it is small enough to live on the stack, otherwise
/// it allocates a HashMap on the heap
///
/// Hashing can be slower than just iterating through an array
/// if the array is small, which is where it makes most sense
#[derive(Debug, Clone)]
pub struct InlineHashMap<K, V, const N: usize>(InlineHashMapInner<K, V, N>);

impl<K, V, const N: usize> InlineHashMap<K, V, N>
where
    K: Hash + Eq,
{
    /// Creates a new InlineHashMap
    pub(crate) fn new() -> Self {
        Self(InlineHashMapInner::new())
    }

    /// Returns the number of elements in the map
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// If `self` is inlined, this returns the underlying raw parts that make up this `InlineHashMap`.
    ///
    /// Only the first `.1` elements are initialized.
    #[inline]
    pub fn inline_parts_mut(&mut self) -> Option<(&mut [MaybeUninit<(K, V)>; N], usize)> {
        self.0.inline_parts_mut()
    }

    /// Copies `self` into a new `HashMap<K, V>`
    #[inline]
    pub fn to_map(&self) -> HashMap<K, V>
    where
        K: Clone + Hash + Eq,
        V: Clone,
    {
        self.0.to_map()
    }

    /// Checks whether this vector is allocated on the heap
    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        self.0.is_heap_allocated()
    }

    /// Inserts a new element into the map
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value)
    }

    /// Removes an element from the map, and returns ownership over the value
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.get_mut(key)
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }
}

enum InlineHashMapInner<K, V, const N: usize> {
    Inline {
        len: usize,
        data: [MaybeUninit<(K, V)>; N],
    },
    Heap(HashMap<K, V>),
}

impl<K, V, const N: usize> Debug for InlineHashMapInner<K, V, N>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InlineHashMap<{} items>", self.len())
    }
}

impl<K, V, const N: usize> Clone for InlineHashMapInner<K, V, N>
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

impl<K, V, const N: usize> Drop for InlineHashMapInner<K, V, N> {
    fn drop(&mut self) {
        if let Some((data, len)) = self.inline_parts_mut() {
            for element in data.iter_mut().take(len) {
                unsafe { ptr::drop_in_place(element.as_mut_ptr()) };
            }
        }
    }
}

impl<K, V, const N: usize> InlineHashMapInner<K, V, N> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self::Inline {
            len: 0,
            data: super::uninit_array(),
        }
    }

    #[inline]
    pub fn inline_parts_mut(&mut self) -> Option<(&mut [MaybeUninit<(K, V)>; N], usize)> {
        match self {
            Self::Heap(_) => None,
            Self::Inline { len, data } => Some((data, *len)),
        }
    }

    #[inline]
    fn to_map(&self) -> HashMap<K, V>
    where
        K: Clone + Hash + Eq,
        V: Clone,
    {
        match &self {
            InlineHashMapInner::Heap(m) => m.clone(),
            InlineHashMapInner::Inline { len, data } => {
                let mut new_data = HashMap::with_capacity(*len);

                let iter = data.into_iter().take(*len);

                for element in iter {
                    let element = unsafe { &*element.as_ptr() };
                    let (key, value) = element.clone();
                    new_data.insert(key, value);
                }

                new_data
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Heap(map) => map.len(),
        }
    }

    #[inline]
    pub fn is_heap_allocated(&self) -> bool {
        matches!(self, Self::Heap(_))
    }
}

impl<K: Eq + Hash, V, const N: usize> InlineHashMapInner<K, V, N> {
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

    pub fn get_mut<'m>(&'m mut self, k: &K) -> Option<&'m mut V> {
        match self {
            Self::Inline { data, len } => unsafe {
                InlineHashMapIteratorMut::new(data, *len)
                    .find(|(key, _)| key.eq(k))
                    .map(|(_, value)| value)
            },
            Self::Heap(map) => map.get_mut(k),
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self {
            Self::Inline { data, len } => {
                let idx = data
                    .iter()
                    .take(*len)
                    .map(|x| unsafe { &*x.as_ptr() })
                    .position(|x| &x.0 == key)?;

                let element = unsafe {
                    std::mem::replace(data.get_unchecked_mut(idx), MaybeUninit::uninit())
                };

                // HashMap order is not guaranteed, so instead of swapping every item like we do with InlineVec,
                // we can simply swap the last item with the one we want to remove.
                data.swap(idx, *len - 1);
                *len -= 1;

                Some(unsafe { element.assume_init().1 })
            }
            Self::Heap(h) => h.remove(key),
        }
    }

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
            let new_heap = Self::Heap(map);

            // do not call the destructor!
            unsafe { ptr::write(self, new_heap) };
            return;
        } else {
            array[*len].write((k, v));
            *len += 1;
        }
    }

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
pub struct InlineHashMapIteratorMut<'a, K, V> {
    array: &'a mut [MaybeUninit<(K, V)>],
    idx: usize,
    len: usize,
}

impl<'a, K, V> InlineHashMapIteratorMut<'a, K, V> {
    pub(crate) unsafe fn new(array: &'a mut [MaybeUninit<(K, V)>], len: usize) -> Self {
        Self { array, idx: 0, len }
    }
}

impl<'a, K, V> Iterator for InlineHashMapIteratorMut<'a, K, V> {
    type Item = &'a mut (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }

        let element = unsafe { &mut *self.array[self.idx].as_mut_ptr() };
        self.idx += 1;

        Some(element)
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
    fn inlinehashmap_remove() {
        let mut x = InlineHashMap::<usize, usize, 4>::new();
        x.insert(789, 1336);
        assert_eq!(x.len(), 1);
        assert_eq!(x.get(&789), Some(&1336));
        assert_eq!(x.remove(&789), Some(1336));
        assert_eq!(x.len(), 0);

        assert_eq!(x.remove(&789), None);

        for i in 0..4 {
            x.insert(i, i * 2);
        }

        assert!(!x.is_heap_allocated());
        assert_eq!(x.len(), 4);

        assert_eq!(x.remove(&2), Some(4));
        assert_eq!(x.len(), 3);

        assert_eq!(x.remove(&3), Some(6));
        assert_eq!(x.len(), 2);

        assert_eq!(x.remove(&1), Some(2));
        assert_eq!(x.len(), 1);

        assert_eq!(x.remove(&0), Some(0));
        assert_eq!(x.len(), 0);
        assert!(!x.is_heap_allocated());

        // trigger heap allocation
        for i in 0..8 {
            x.insert(i, i * 2);
        }
        assert!(x.is_heap_allocated());
        assert_eq!(x.len(), 8);

        assert_eq!(x.remove(&7), Some(14));
        assert_eq!(x.remove(&0), Some(0));
    }

    #[test]
    fn inlinehashmap_remove_heap() {
        let mut x = InlineHashMap::<usize, String, 4>::new();
        x.insert(42, "test".into());
        assert_eq!(x.len(), 1);
        assert_eq!(x.remove(&42), Some("test".into()));
        assert_eq!(x.len(), 0);
    }

    #[test]
    fn inlinehashmap_clone() {
        let mut x = InlineHashMapInner::<usize, usize, 4>::new();

        for i in 0..10 {
            x.insert(i, i * 2);
        }

        let x = x.clone();
        assert_eq!(x.len(), 10);
        assert!(x.is_heap_allocated());
        assert_eq!(x.get(&9), Some(&18));
    }

    #[test]
    fn inlinehashmap_to_map_stack() {
        let mut x = InlineHashMapInner::<usize, usize, 4>::new();

        for i in 0..4 {
            x.insert(i, i * 2);
        }

        assert!(!x.is_heap_allocated());
        assert_eq!(x.len(), 4);

        let xx = x.to_map();
        assert_eq!(xx.get(&0), Some(&0));
        assert_eq!(xx.get(&1), Some(&2));
        assert_eq!(xx.get(&2), Some(&4));
        assert_eq!(xx.get(&3), Some(&6));
        assert_eq!(xx.len(), 4);

        x.insert(42, 1337);
        assert!(x.is_heap_allocated());
        assert_eq!(x.len(), 5);
        assert_eq!(x.get(&42), Some(&1337));

        let xx = x.to_map();
        assert_eq!(xx.get(&0), Some(&0));
        assert_eq!(xx.get(&42), Some(&1337));
        assert_eq!(xx.len(), 5);
    }

    #[test]
    fn inlinehashmap_to_map_heap() {
        let mut x = InlineHashMapInner::<usize, String, 4>::new();

        for i in 0..4 {
            x.insert(i, i.to_string());
        }

        assert!(!x.is_heap_allocated());
        assert_eq!(x.len(), 4);

        let xx = x.to_map();
        assert_eq!(&*xx[&0], "0");
        assert_eq!(&*xx[&1], "1");
        assert_eq!(&*xx[&2], "2");
        assert_eq!(&*xx[&3], "3");
        assert_eq!(xx.len(), 4);

        x.insert(42, "1337".into());
        assert!(x.is_heap_allocated());
        assert_eq!(x.len(), 5);
        assert_eq!(x.get(&42).map(|x| &**x), Some("1337"));

        let xx = x.to_map();
        assert_eq!(&*xx[&0], "0");
        assert_eq!(&*xx[&42], "1337");
        assert_eq!(xx.len(), 5);
    }

    #[test]
    fn inlinehashmap_drop_stack() {
        let mut x = InlineHashMapInner::<usize, String, 4>::new();

        for i in 0..3 {
            x.insert(i, i.to_string());
        }

        assert_eq!(x.len(), 3);
        assert!(!x.is_heap_allocated());
    }

    #[test]
    fn inlinehashmap_drop_heap() {
        let mut x = InlineHashMapInner::<usize, String, 4>::new();

        for i in 0..16 {
            x.insert(i, i.to_string());
        }

        assert_eq!(x.len(), 16);
        assert!(x.is_heap_allocated());
    }

    #[test]
    fn inlinehashmap() {
        let mut x = InlineHashMapInner::<&'static str, usize, 4>::new();
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
