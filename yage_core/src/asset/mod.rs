use core::num::NonZero;
use core::future::Future;
use alloc::sync::Arc;


pub trait Loader<I, N, C: Cache<NonZero<usize>>>: Sized {
    type Error;

    type LoadFuture<'a>: Future<Output = Result<CowHandle<'a, C>, Self::Error>> + 'a
    where
        Self: 'a, C: 'a;

    type InitFuture: Future<Output = Result<Self, Self::Error>>;

    fn init(init: I) -> Self::InitFuture
    where
        Self: Sized;

    fn load(&self, name: N) -> Self::LoadFuture<'_>;
}



pub trait Key {
  type Value;
}

macro_rules! bkey {
    ($k:ty => $value:ty) => {
        impl Key for $k {
          type Value = $value;
        }

        impl Key for $value {
          type Value = $k;
        }
    };
    ($($k:ty => $v:ty)*) => {
      $(
        bkey!($k => $v);
      )*
    }
}

bkey! {
  NonZero<usize> => Asset<alloc::boxed::Box<[u8]>>
}


pub trait Cache<I: Key> {
  
  fn lookup(&self, index: &I) -> Option<&I::Value>;
  fn insert(&self, index: &I, value: I::Value) -> bool;
  fn remove(&self, index: &I) -> Option<(I, I::Value)>;

  // clones an asset entry, should only be called by this crate (maybe?)
  fn clone_entry(&self, index: &I, token: crate::machine_cog::sealed::OnlyCalledByThisCrate) -> I;
}

pub enum AssetKind {
  JsonBody,
  GameWorld,
  RawData,
  Mesh,
  Other
}

pub struct Asset<B> {
  pub kind: AssetKind,
  pub(crate) gen: usize, // used so we can uniquely hash every single time
  pub(crate) data: B,
}

pub struct OwnedHandle<L: Cache<NonZero<usize>>> {
  index: NonZero<usize>,
  cache: Arc<L>
}
/// this can be slightly more effficient, because now it borrows the `Arc` instead of cloning it (eliding an atomic op)
pub struct BorrowedHandle<'a, L>
{
  index: NonZero<usize>,
  cache: &'a Arc<L>,
}

impl<L> Copy for BorrowedHandle<'_, L> {}
impl<L> Clone for BorrowedHandle<'_, L> {
  fn clone(&self) -> Self {
    *self  
  }
}

impl<L> BorrowedHandle<'_, L>
where 
  L: Cache<NonZero<usize>>
{

  pub fn to_owned_handle(self) -> OwnedHandle<L> {
    let token = crate::token!();
    let new_index = self.cache.clone_entry(&self.index, token);
    OwnedHandle {
      cache: self.cache.clone(),
      index: new_index
    }
  }  
}

impl<L> OwnedHandle<L> 
where
  L: Cache<NonZero<usize>>
{}

impl<L> Drop for OwnedHandle<L> 
where
  L: Cache<NonZero<usize>>
{

  fn drop(&mut self) {
    assert!(self.cache.remove(&self.index).is_some(), "this should be still in there")  
  }
  
}

pub enum CowHandle<'a, C: Cache<NonZero<usize>>> {
  Borrowed(BorrowedHandle<'a, C>),
  Owned(OwnedHandle<C>)
}