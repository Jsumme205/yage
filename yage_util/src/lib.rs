#![cfg_attr(not(feature = "std"), no_std)]

use core::mem::MaybeUninit;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

pub mod inline;

pub mod atomic;
pub mod container_trait;
pub mod list;
pub mod testing;

pub trait IterExt: Iterator {
    fn next_chunk<const N: usize>(&mut self) -> Option<[Self::Item; N]>
    where
        Self: Sized,
    {
        let mut arr = ArrayBuilder::new();
        for _ in 0..N {
            arr.push(self.next()?);
        }
        arr.take()
    }
}

impl<I> IterExt for I where I: Iterator {}

struct ArrayBuilder<T, const N: usize> {
    elem: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayBuilder<T, N> {
    pub fn new() -> Self {
        Self {
            elem: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    pub fn push(&mut self, elem: T) {
        let place = &mut self.elem[self.len];
        place.write(elem);
        self.len += 1;
    }

    pub fn take(&mut self) -> Option<[T; N]> {
        if self.len == N {
            self.len = 0;
            let arr = core::mem::replace(&mut self.elem, [const { MaybeUninit::uninit() }; N]);
            // SAFETY: despite what rust thinks about transmuting, this is completely safe
            return Some(unsafe { (&raw const arr as *const [T; N]).read() });
        }
        None
    }
}
