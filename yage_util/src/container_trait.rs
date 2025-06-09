//use crate::atomic::{AtomicMut, AtomicRef};
use core::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
};

/// a marker trait that shows that a certain type could represent another item
///
///
/// for example:
///   an implementation could be used on a `AtomicRef<'a, T>` like so:
///
///     ```
///         impl<'a, T> Represents<&'a T> for AtomicRef<'a, T> {}
///     ```
///
/// this allows flexibilty within the `Container` trait while still restricting types
///
/// # safety:
///
/// any type that implements this MUST
/// accurately represent `T`
pub unsafe trait Represents<T> {}

unsafe impl<T> Represents<T> for T {}
unsafe impl<T> Represents<&T> for *const T {}
unsafe impl<T> Represents<&mut T> for *mut T {}
//unsafe impl<'a, T> Represents<&'a T> for AtomicRef<'a, T> {}
//unsafe impl<'a, T> Represents<&'a mut T> for AtomicMut<'a, T> {}
//unsafe impl<'a, T> Represents<&'a T> for AtomicMut<'a, T> {}
unsafe impl<'a, T: Copy> Represents<&'a mut T> for &'a Cell<T> {}
unsafe impl<'a, T> Represents<&'a mut T> for &'a UnsafeCell<T> {}
unsafe impl<'a, T: Copy> Represents<&'a T> for &'a Cell<T> {}
unsafe impl<'a, T> Represents<&'a T> for &'a UnsafeCell<T> {}

macro_rules! impl_for_tuples {
    ($($ty:tt)*) => {
        unsafe impl<'a, $($ty),*> Represents<&'a ($($ty),*)> for ($(&'a $ty),*) {}
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

impl<C, T> Container<T> for C
where
    C: DerefMut<Target = [T]>,
{
    type Mutable<'a>
        = core::slice::IterMut<'a, T>
    where
        T: 'a,
        C: 'a;
    type Iterator<'a>
        = core::slice::Iter<'a, T>
    where
        T: 'a,
        C: 'a;

    fn iterator(&self) -> Self::Iterator<'_> {
        <[T]>::iter(&*self)
    }

    fn mutable_iterator(&mut self) -> Self::Mutable<'_> {
        <[T]>::iter_mut(&mut *self)
    }
}
