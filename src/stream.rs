use std::cmp::{min, max};
use std::cmp::PartialEq;

pub struct Stream<'a, T> {
    pub idx: usize,
    data: &'a [T]
}

impl<'a, T: Copy> Stream<'a, T> {
    pub fn current_cpy(&self) -> Option<T> {
        self.data.get(self.idx).copied()
    }
}

impl<'a, T> Stream<'a, T> {
    pub fn new(data: &'a [T]) -> Stream<T> {
        Self { data, idx: 0 }
    }

    pub fn current(&self) -> Option<&T> {
        self.data.get(self.idx)
    }

    pub fn current_unchecked(&self) -> &T {
        &self.data[self.idx]
    }

    pub fn is_eof(&self) -> bool {
        self.idx >= self.data.len()
    }

    pub fn slice_unchecked(&self, from: usize, to: usize) -> &[T] {
        &self.data[from..to]
    }

    pub fn slice(&self, from: usize, to: usize) -> &[T] {
        &self.data[from..min(self.data.len(), to)]
    }
}