use std::mem::MaybeUninit;

/// Inline HashMap
pub mod hashmap;

fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    // SAFETY: an array of MaybeUninits is allowed to be entirely uninit
    unsafe { MaybeUninit::uninit().assume_init() }
}
