#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

pub mod atomic;
pub mod container_trait;
pub mod list;
pub mod testing;
