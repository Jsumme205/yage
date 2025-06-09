#![no_std]

/// bidirectional hashmap, that unlike other bimaps only instansiates one map
/// this does mean that we have to implement a map from the bottom up, but the performance i believe will be worth it

use core::hash::{BuildHasher, Hash};
use core::borrow::Borrow;

extern crate alloc;

mod raw;
pub(crate) mod ctl;

pub struct RandomState;

impl RandomState {

  pub const fn new() -> Self {
    RandomState
  }
  
}

fn make_hash<Q, S>(hasher: &S, key: &Q) -> u64
where 
    S: BuildHasher,
    Q: Hash + ?Sized
{
  hasher.hash_one(key)
}

pub struct Bimap<L, R, LS = RandomState, RS = RandomState> {
  left_hash: LS,
  right_hash: RS,
  raw: raw::Table<(L, R)>,
}

impl<L, R> Bimap<L, R> {

  pub fn new() -> Self {
    Self {
      left_hash: RandomState::new(),
      right_hash: RandomState::new(),
      raw: raw::Table::new(),
    }
  }

  pub fn get_right<Q>(&self, key: &Q) -> Option<&R> 
  where   
    L: Borrow<Q>,
    Q: Hash + Eq + ?Sized
  {
    None
  }

  pub fn get_left<Q>(&self, key: &Q) -> Option<&L> 
  where 
    R: Borrow<Q>,
    Q: Hash + Eq + ?Sized
  {
    None
  }
  
}