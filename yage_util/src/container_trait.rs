use core::str::Bytes;

use crate::atomic::{AtomicMut, AtomicRef};
use alloc::{string::String, vec::Vec};

/// a marker trait that shows that a certain type could represent another item
/// for example:
///   an impllementation could be used on a `AtomicRef<'a, T>` like so:
///
///     ```
///         impl<'a, T> Represents<&'a T> for AtomicRef<'a, T> {}
///     ```
///
/// this allows flexibilty within the `Container` trait while still restricting types
pub trait Represents<T> {}

impl<T> Represents<T> for T {}
impl<T> Represents<&T> for *const T {}
impl<T> Represents<&mut T> for *mut T {}
impl<'a, T> Represents<&'a T> for AtomicRef<'a, T> {}
impl<'a, T> Represents<&'a mut T> for AtomicMut<'a, T> {}
impl<'a, T> Represents<&'a T> for AtomicMut<'a, T> {}

macro_rules! impl_for_tuples {
    ($($ty:tt)*) => {
        impl<'a, $($ty),*> Represents<&'a ($($ty),*)> for ($(&'a $ty),*) {}
    };
}

impl_for_tuples!(A B);
impl_for_tuples!(A B C);
impl_for_tuples!(A B C D);
impl_for_tuples!(A B C D E);
impl_for_tuples!(A B C D E F);

/// a generic container type
/// this allows a generic container access for the `System` trait
pub trait Container<Item> {
    /// iterator that takes `Item` by an immutable reference
    /// NOTE: this could also take any item that represents `&'a Item`,
    /// such as AtomicRef<'a, Item> or AtomicMut<'a, Item>
    type Iterator<'a>: Iterator<Item: Represents<&'a Item>>
    where
        Self: 'a,
        Item: 'a;
    type Mutable<'a>: Iterator<Item: Represents<&'a mut Item>>
    where
        Self: 'a,
        Item: 'a;

    fn iterator(&self) -> Self::Iterator<'_>;

    fn mutable_iterator(&mut self) -> Self::Mutable<'_>;
}

impl<T> Container<T> for Vec<T> {
    type Iterator<'a>
        = core::slice::Iter<'a, T>
    where
        T: 'a;
    type Mutable<'a>
        = core::slice::IterMut<'a, T>
    where
        T: 'a;

    fn iterator(&self) -> Self::Iterator<'_> {
        self.iter()
    }

    fn mutable_iterator(&mut self) -> Self::Mutable<'_> {
        self.iter_mut()
    }
}

impl<T> Container<T> for [T] {
    type Iterator<'a>
        = core::slice::Iter<'a, T>
    where
        T: 'a;
    type Mutable<'a>
        = core::slice::IterMut<'a, T>
    where
        T: 'a;

    fn iterator(&self) -> Self::Iterator<'_> {
        self.iter()
    }

    fn mutable_iterator(&mut self) -> Self::Mutable<'_> {
        self.iter_mut()
    }
}
