use std::hash::{Hash, Hasher};
use std::num::NonZero;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use yage_core::asset::Cache;

/// this allows us to unconditionally implement `Hash`, `Eq`, and `PartialEq`
/// this takes account for nothing on the data; as long as other.id == self.id (and by association, other.id != self.id), these are considered equal
pub struct IdHashed<T> {
    id: NonZero<u32>,
    data: T,
}

impl<T> IdHashed<T> {
    pub const fn new(data: T) -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        let id = unsafe { NonZero::new_unchecked(NEXT_ID.fetch_add(1, Ordering::Relaxed)) };
        Self { id, data }
    }
}

impl<T> PartialEq for IdHashed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for IdHashed<T> {}

impl<T> Hash for IdHashed<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

pub struct AssetCache {}

impl Cache<NonZero<usize>> for AssetCache {
    fn clone_entry(
        &self,
        index: &NonZero<usize>,
        token: yage_core::machine_cog::sealed::OnlyCalledByThisCrate,
    ) -> NonZero<usize> {
        todo!()
    }

    fn insert(
        &self,
        index: &NonZero<usize>,
        value: <NonZero<usize> as yage_core::asset::Key>::Value,
    ) -> bool {
        todo!()
    }

    fn lookup(
        &self,
        index: &NonZero<usize>,
    ) -> Option<&<NonZero<usize> as yage_core::asset::Key>::Value> {
        todo!()
    }

    fn remove(
        &self,
        index: &NonZero<usize>,
    ) -> Option<(
        NonZero<usize>,
        <NonZero<usize> as yage_core::asset::Key>::Value,
    )> {
        todo!()
    }
}
