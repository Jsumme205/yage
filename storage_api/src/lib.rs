use std::{hash::Hash, mem::MaybeUninit, ptr::NonNull};

pub trait PointedTo {
    type Metadata: Copy + Clone + PartialEq + PartialOrd + Hash;

    unsafe fn from_raw_parts(data: NonNull<()>, metadata: Self::Metadata) -> NonNull<Self>;

    fn metadata(this: NonNull<Self>) -> Self::Metadata;
}

impl<T> PointedTo for T {
    type Metadata = ();

    fn metadata(this: NonNull<Self>) -> Self::Metadata {
        let _ = this;
        ()
    }

    unsafe fn from_raw_parts(data: NonNull<()>, metadata: Self::Metadata) -> NonNull<Self> {
        let _ = metadata;
        data.cast()
    }
}

impl<T> PointedTo for [T] {
    type Metadata = usize;

    fn metadata(this: NonNull<Self>) -> Self::Metadata {
        this.len()
    }

    unsafe fn from_raw_parts(data: NonNull<()>, metadata: Self::Metadata) -> NonNull<Self> {
        NonNull::slice_from_raw_parts(data.cast(), metadata)
    }
}

/// basically means that this can point to uninitialized data.
pub unsafe trait UninitSafe {}

unsafe impl<T: ?Sized> UninitSafe for NonNull<T> {}
unsafe impl<T: ?Sized> UninitSafe for *mut T {}
unsafe impl<T: ?Sized> UninitSafe for *const T {}

impl<T: ?Sized> AsRaw<T> for NonNull<T> {
    fn as_ptr(&self) -> *const T {
        NonNull::as_ptr(*self)
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        NonNull::as_ptr(*self)
    }
}

pub trait AsRaw<T: ?Sized> {
    fn as_ptr(&self) -> *const T;

    fn as_mut_ptr(&mut self) -> *mut T;
}

pub unsafe trait Storage {
    type Handle<T: ?Sized + PointedTo>: Clone + Copy + UninitSafe + AsRaw<T>;

    fn dangling<T>() -> Self::Handle<T>;

    unsafe fn allocate<T>(&mut self, val: T) -> Option<Self::Handle<T>>
    where
        T: PointedTo;

    unsafe fn deallocate<T>(&mut self, handle: Self::Handle<T>)
    where
        T: PointedTo + ?Sized;

    unsafe fn resolve<T>(&self, handle: Self::Handle<T>) -> NonNull<u8>
    where
        T: PointedTo + ?Sized;

    unsafe fn allocate_unsize<F, T, U>(&mut self, init: T, op: F) -> Self::Handle<U>
    where
        T: PointedTo,
        F: FnOnce(Self::Handle<T>) -> Self::Handle<U>,
        U: PointedTo + ?Sized;
}

pub unsafe trait ArrayStorage: Storage {
    unsafe fn array<T>(&mut self, len: usize) -> Self::Handle<[MaybeUninit<T>]>
    where
        [MaybeUninit<T>]: PointedTo<Metadata = usize>;

    unsafe fn grow<T>(
        &mut self,
        old: Self::Handle<[MaybeUninit<T>]>,
        new_len: usize,
    ) -> Self::Handle<[MaybeUninit<T>]>;
}

pub unsafe trait PinnedStorage: Storage {}
