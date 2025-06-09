
#[cfg(target_arch = "x86")]
use core::arch::x86;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;

pub(crate) struct Group(x86::__m128i);
pub(crate) type Tag = u8;


impl Group {

  pub(crate) const WIDTH: usize = core::mem::size_of::<Self>();

  pub(crate) const fn static_empty() -> &'static [Tag; Group::WIDTH] {
    #[repr(C)]
    struct Aligned {
      _align: [Group; 0],
      tags: [Tag; Group::WIDTH]
    }

    const TAGS: Aligned = Aligned {
      _align: [],
      tags: [!0; Group::WIDTH]
    };
    &TAGS.tags
  }
  
}