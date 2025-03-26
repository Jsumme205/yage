use std::{alloc::Layout, ptr::NonNull};

pub unsafe trait Allocator {
    type Error;

    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, Self::Error>;

    fn reallocate(
        &mut self,
        old_ptr: *mut u8,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, Self::Error>;

    fn deallocate(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), Self::Error>;
}
