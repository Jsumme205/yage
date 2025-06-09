use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

mod __sealed {

    pub trait Sealed {}

    impl<T, const N: usize> Sealed for [T; N] {}
}

pub trait Array: __sealed::Sealed {
    type Element;
    type MaybeUninit;
    type Key;
}

pub enum Len<const N: usize> {}

impl<T, const N: usize> Array for [T; N] {
    type Element = T;
    type Key = Len<N>;
    type MaybeUninit = [MaybeUninit<T>; N];
}

pub struct Ssv<T, const N: usize> {
    inner: SsvInner<[T; N]>,
    initialized: usize,
}

union SsvInner<A: Array> {
    stack: ManuallyDrop<Stack<A>>,
    heap: Heap<A::Element>,
}

struct Heap<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> Copy for Heap<T> {}
impl<T> Clone for Heap<T> {
    fn clone(&self) -> Self {
        *self
    }
}

struct Stack<A: Array> {
    array: A::MaybeUninit,
}
