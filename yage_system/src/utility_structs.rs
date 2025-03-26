use std::{marker::PhantomData, ptr::NonNull};
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
        }

        TransformSystem(f, PhantomData)
    }
}

pub struct Entity {
    pub(crate) id: u32,
}

pub struct Header {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) len: u32,
    pub(crate) cap: u32,
}

impl Header {
    pub(crate) const DANGLING: Self = Self {
        ptr: NonNull::dangling(),
        len: 0,
        cap: 0,
    };
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

    pub(crate) const unsafe fn as_slice(&self, len: u32) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), len as _) }
    }

    pub(crate) const unsafe fn as_mut_slice(&mut self, len: u32) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), len as _) }
    }
}
