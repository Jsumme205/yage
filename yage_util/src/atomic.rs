use core::cell::{Cell, RefCell, UnsafeCell};

use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

mod borrow;

pub trait Counter {
    const INIT: Self;

    fn increment(&self);
    fn decrement(&self);

    fn set_mutable(&self) -> bool;

    fn is_set_mutable(&self) -> bool;
}

impl Counter for Cell<usize> {
    const INIT: Self = Cell::new(0);

    fn decrement(&self) {
        debug_assert!(!self.is_set_mutable());
        self.set(self.get() - 1);
    }

    fn increment(&self) {
        debug_assert!(!self.is_set_mutable());
        self.set(self.get() + 1);
    }

    fn set_mutable(&self) -> bool {
        if self.get() != 0 {
            return false;
        }
        self.set(usize::MAX);
        true
    }

    fn is_set_mutable(&self) -> bool {
        self.get() == usize::MAX
    }
}

impl Counter for AtomicUsize {
    const INIT: Self = AtomicUsize::new(0);

    fn decrement(&self) {
        debug_assert!(!self.is_set_mutable());
        self.fetch_sub(1, core::sync::atomic::Ordering::AcqRel);
    }

    fn increment(&self) {
        debug_assert!(!self.is_set_mutable());
        self.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
    }

    fn set_mutable(&self) -> bool {
        let val = self.load(core::sync::atomic::Ordering::Acquire);
        if val != 0 {
            return false;
        }
        self.store(usize::MAX, core::sync::atomic::Ordering::Release);
        true
    }

    fn is_set_mutable(&self) -> bool {
        self.load(core::sync::atomic::Ordering::Acquire) == usize::MAX
    }
}

pub struct Mutable<C, T: ?Sized> {
    borrow: C,
    val: UnsafeCell<T>,
}

unsafe impl<C: Send + Sync, T: ?Sized + Send> Send for Mutable<C, T> {}
unsafe impl<C: Send + Sync, T: ?Sized + Send> Sync for Mutable<C, T> {}

impl<C, T> Mutable<C, T>
where
    C: Counter,
{
    pub const fn new(val: T) -> Self {
        Self {
            borrow: C::INIT,
            val: UnsafeCell::new(val),
        }
    }
}

pub struct Ref<'a, C, T: ?Sized> {
    value: NonNull<T>,
    borrow: &'a C,
}
