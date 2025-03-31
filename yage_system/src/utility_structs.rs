use std::{
    alloc::Layout,
    cell::Cell,
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    u32,
    u64::MAX,
};
use yage_util::container_trait::Container;

use crate::System;

pub struct Vec3(pub f32, pub f32, pub f32);

impl Vec3 {
    /// creates a transform callback over a collection of `Vec3`'s
    pub fn transform<F, C>(f: F) -> impl System<Self, Collection = C>
    where
        F: FnMut(&mut C),
        C: Container<Self>,
    {
        struct TransformSystem<F, C>(F, PhantomData<C>);

        impl<F, C> System<Vec3> for TransformSystem<F, C>
        where
            F: FnMut(&mut C),
            C: Container<Vec3>,
        {
            type Collection = C;

            fn run_system(&mut self, collection: &mut Self::Collection) {
                (self.0)(collection)
            }

            fn consume_iter(&mut self, iter: <Self::Collection as Container<Vec3>>::Mutable<'_>) {
                todo!()
            }
        }

        TransformSystem(f, PhantomData)
    }
}

pub struct Entity {
    // layout:
    // bits 0..31: id (index)
    // bits 32..62: reference count
    // bit 63: 0 if used, 1 if null
    pub(crate) id: NonNull<AtomicU64>,
}

impl Entity {
    const NULL_MASK: u64 = 1 << (u64::BITS - 1);

    pub fn set_null(&self) {
        unsafe {
            self.id
                .as_ref()
                .fetch_or(Self::NULL_MASK, Ordering::Relaxed);
        }
    }

    pub fn set_occupied(&self) {
        unsafe {
            self.id
                .as_ref()
                .fetch_and(!Self::NULL_MASK, Ordering::Relaxed);
        }
    }
}

pub struct Header<Layout: GetLayout> {
    pub(crate) len: u32,
    pub(crate) cap: u32,
    pub(crate) layout: &'static Layout,
}

pub trait GetLayout: 'static {
    fn layout(&self) -> Layout;
}

impl<L: GetLayout> Header<L> {
    pub const fn new(is_constant: bool, cap: u32, layout: &'static L) -> Self {
        assert!(
            cap <= i32::MAX as _,
            "capacity cannot exceed i32::MAX, we have to save that upper bit for a flag"
        );
        let mask: u32 = if is_constant {
            (1 << (u32::BITS - 1)) as u32
        } else {
            0
        };
        Self {
            len: 0,
            cap: mask | cap,
            layout,
        }
    }

    pub fn layout(&self) -> Layout {
        self.layout.layout()
    }

    #[inline]
    pub const fn capacity(&self) -> u32 {
        (self.cap << 1) >> 1
    }

    pub const fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub const fn is_constant(&self) -> bool {
        (self.cap >> (u32::BITS - 1)) != 0
    }

    pub const fn is_full(&self) -> bool {
        self.capacity() == self.len
    }

    pub const unsafe fn set_len(&mut self, new_len: u32) {
        self.len = new_len;
    }

    pub const fn raw_layout(&self) -> &'static L {
        self.layout
    }
}

impl<Layout: GetLayout> Drop for Header<Layout> {
    fn drop(&mut self) {
        if !self.is_constant() {
            // if we did leak, we should drop it
            drop(unsafe { Box::from_raw(self.layout as *const _ as *mut Layout) });
        }
    }
}

/// slice where the length is stored elsewhere
pub struct ThinSlice<T> {
    pub(crate) ptr: NonNull<T>,
    _marker: PhantomData<[T]>,
}

impl<T> ThinSlice<T> {
    pub(crate) const DANGLING: Self = Self {
        ptr: NonNull::dangling(),
        _marker: PhantomData,
    };

    pub const unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) const unsafe fn as_slice(&self, len: u32) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), len as _) }
    }

    #[inline(always)]
    pub(crate) const unsafe fn as_mut_slice(&mut self, len: u32) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), len as _) }
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub const unsafe fn get(&self, idx: usize) -> &T {
        unsafe { self.ptr.add(idx).as_ref() }
    }

    pub const unsafe fn get_mut(&mut self, idx: usize) -> &mut T {
        unsafe { self.ptr.add(idx).as_mut() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLayout;
    impl GetLayout for DummyLayout {
        fn layout(&self) -> Layout {
            Layout::new::<u32>()
        }
    }

    #[test]
    fn test_header_correctness() {
        let header = Header::new(true, 3, &DummyLayout);
        assert!(header.is_constant());
        assert!(header.capacity() == 3);
    }
}
