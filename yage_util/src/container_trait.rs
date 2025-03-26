use alloc::vec::Vec;

/// a generic container type
/// this allows a generic container access for the `System` trait
pub trait Container<Item> {
    type Iterator<'a>: Iterator<Item = &'a Item>
    where
        Self: 'a,
        Item: 'a;
    type Mutable<'a>: Iterator<Item = &'a mut Item>
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
