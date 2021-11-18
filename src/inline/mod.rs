use std::mem::MaybeUninit;

pub mod hashmap;
pub mod vec;

fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    // SAFETY: an array of MaybeUninits is allowed to be entirely uninit
    unsafe { MaybeUninit::uninit().assume_init() }
}
