use std::cmp::min;

/// Internal struct for iterating over input bytes
#[derive(Debug)]
pub struct Stream<'a, T> {
    pub idx: usize,
    data: &'a [T],
}

impl<'a, T: Copy> Stream<'a, T> {
    /// Returns a copy of the current element
    pub fn current_cpy(&self) -> Option<T> {
        self.data.get(self.idx).copied()
    }
    /// Returns a copy of the current element, and panicks if it's out of bounds
    pub fn current_unchecked_cpy(&self) -> T {
        self.data[self.idx]
    }
    /// Returns a copy of the next element
    pub fn next_cpy(&mut self) -> Option<T> {
        self.idx += 1;
        self.data.get(self.idx).copied()
    }
}

impl<'a, T: Eq + Copy> Stream<'a, T> {
    /// Increases internal index by 1 if the given element matches the current element
    /// If it does match, the expected character is returned
    pub fn expect_and_skip(&mut self, expect: T) -> Option<T> {
        self.expect_oneof_and_skip(&[expect])
    }

    pub fn expect_oneof_and_skip(&mut self, expect: &[T]) -> Option<T> {
        let c = self.current_cpy()?;

        if expect.contains(&c) {
            self.idx += 1;
            return Some(c);
        }

        None
    }

    /// Same as expect_and_skip, but returns a bool
    pub fn expect_and_skip_cond(&mut self, expect: T) -> bool {
        self.expect_and_skip(expect)
            .map(|c| c == expect)
            .unwrap_or(false)
    }
}

impl<'a, T> Stream<'a, T> {
    /// Creates a new stream
    pub fn new(data: &'a [T]) -> Stream<T> {
        Self { data, idx: 0 }
    }

    /// Returns the current element
    pub fn current(&self) -> Option<&T> {
        self.data.get(self.idx)
    }

    /// Returns the next element
    pub fn next(&mut self) -> Option<&T> {
        self.data.get(self.idx + 1).map(|c| {
            self.idx += 1;
            c
        })
    }

    /// Returns the current element, but panicks if out of bounds
    pub fn current_unchecked(&self) -> &T {
        &self.data[self.idx]
    }

    /// Returns the current element without any boundary checks
    /// This *will* cause UB if the current index is out of bounds
    pub unsafe fn current_unchecked_fast(&self) -> &T {
        &*self.data.as_ptr().add(self.idx)
    }

    /// Checks whether the stream has reached the end
    pub fn is_eof(&self) -> bool {
        self.idx >= self.data.len()
    }

    /// Returns a subslice of this stream, and panicks if out of bounds
    pub fn slice_unchecked(&self, from: usize, to: usize) -> &'a [T] {
        &self.data[from..to]
    }

    /// Returns a subslice of this stream but also checks stream length
    /// to prevent out of bounds panicking
    pub fn slice(&self, from: usize, to: usize) -> &'a [T] {
        &self.data[from..min(self.data.len(), to)]
    }

    /// Same as slice, but the second argument is how many elements to slice
    pub fn slice_len(&self, from: usize, len: usize) -> &'a [T] {
        self.slice(from, self.idx + len)
    }

    /// Same as slice, but uses the current index + 1 as `to`
    pub fn slice_from(&self, from: usize) -> &'a [T] {
        self.slice(from, self.idx)
    }
}
