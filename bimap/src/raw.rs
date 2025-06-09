use core::{ptr::NonNull, marker::PhantomData};
use crate::ctl::Group;
use core::alloc::Layout;

#[derive(Clone, Copy)]
struct TableLayout {
  size: usize,
  ctrl_align: usize,
}

impl TableLayout {

  const fn new_for<T>() -> Self {
    let layout = std::alloc::Layout::new::<T>();

    Self {
      size: layout.size(),
      ctrl_align: if layout.align() > Group::WIDTH {
        layout.align()
      } else {
        Group::WIDTH
      }
    }
  }

  fn layout_for(self, buckets:usize) -> Option<(Layout, usize)> {
    let TableLayout { size, ctrl_align } = self;
    // Manual layout calculation since Layout methods are not yet stable.
    let ctrl_offset =
        size.checked_mul(buckets)?.checked_add(ctrl_align - 1)? & !(ctrl_align - 1);
    let len = ctrl_offset.checked_add(buckets + Group::WIDTH)?;

    // We need an additional check to ensure that the allocation doesn't
    // exceed `isize::MAX` (https://github.com/rust-lang/rust/pull/95295).
    if len > isize::MAX as usize - (ctrl_align - 1) {
        return None;
    }

    Some((
        unsafe { Layout::from_size_align_unchecked(len, ctrl_align) },
        ctrl_offset,
    ))
  }
  
}

type HashFn = dyn Fn(&mut RawTable, usize) -> u64;

struct RawTable {
  bucket_mask: usize,
  ctrl: NonNull<u8>,
  growth_left: usize,
  items: usize,
}

impl RawTable {

  const fn new() -> Self {
    Self {
      ctrl: unsafe {
        NonNull::new_unchecked(Group::static_empty().as_ptr().cast_mut())
      },
      bucket_mask: 0,
      items: 0,
      growth_left: 0
    }
  }

  unsafe fn reserve_rehash_inner(
    &mut self,
    additional: usize,
    hasher: &HashFn,
    layout: TableLayout,
    drop: Option<unsafe fn(*mut u8)>
  ) -> Result<(), ()>
  {
    let new_items = match self.items.checked_add(additional) {
      Some(it) => it,
      None => return Err(()),
    };
    let full_cap = bucket_mask_to_capacity(self.bucket_mask);
    if new_items <= full_cap / 2 {
      unsafe { self.__rehash_in_place(hasher, layout.size, drop) };
      Ok(())
    } else {
     unsafe { 
       self.__resize_inner(
         usize::max(new_items, full_cap + 1),
         hasher,
         layout
       )
     }
    }
  }

  unsafe fn __rehash_in_place(
    &mut self, 
    hasher: &HashFn,
    size: usize,
    drop: Option<unsafe fn(*mut u8)>
  ) {}

  unsafe fn __resize_inner(
    &mut self,
    cap: usize,
    hasher: &HashFn,
    layout: TableLayout
  ) -> Result<(), ()>
  {
    Ok(())
  }
}

fn bucket_mask_to_capacity(bucket_mask: usize) -> usize {
    if bucket_mask < 8 {
        // For tables with 1/2/4/8 buckets, we always reserve one empty slot.
        // Keep in mind that the bucket mask is one less than the bucket count.
        bucket_mask
    } else {
        // For larger tables we reserve 12.5% of the slots as empty.
        ((bucket_mask + 1) / 8) * 7
    }
}

pub(super)  struct Table<T> {
  inner: RawTable,
  _marker: PhantomData<T>
}

impl<T> Table<T> {
  const LAYOUT: TableLayout = TableLayout::new_for::<T>();

  pub const fn new() -> Self {
    Self {
      inner: RawTable::new(),
      _marker: PhantomData
    }
  }

  pub fn data_end(&self) -> NonNull<T> {
    self.inner.ctrl.cast()
  }

  //pub fn capacity(&self, table_layout: TableLayout) -> 
  
}
