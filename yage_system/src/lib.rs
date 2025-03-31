use core::convert::Infallible;
use yage_core::allocator_api::Allocator;
use yage_util::container_trait::Container;

mod example;
pub mod macro_impl;
mod system_impls;
pub mod utility_structs;

pub use utility_structs::*;

/// the core trait for systems
/// systems can be thought of as a function that takes an input `In`, and does an operation on it
pub trait System<In> {
    type Collection: Container<In>;

    /// this is meant to run in bulk, for potential optimizations. by default this is already implemented though
    fn run_system(&mut self, collection: &mut Self::Collection) {
        self.consume_iter(collection.mutable_iterator());
    }

    fn consume_iter(&mut self, iter: <Self::Collection as Container<In>>::Mutable<'_>);
}

pub trait FailableSystem<In> {
    type Collection: Container<In>;
    type Error;

    fn run_system(&mut self, collection: &mut Self::Collection) -> Result<(), Self::Error>;
}

impl<T, In> FailableSystem<In> for T
where
    T: System<In> + ?Sized,
{
    type Collection = T::Collection;
    type Error = Infallible;

    fn run_system(&mut self, collection: &mut Self::Collection) -> Result<(), Self::Error> {
        Ok(System::run_system(self, collection))
    }
}

pub trait Systems<Ins> {
    type Collections;

    fn run_systems(&mut self, collections: &mut Self::Collections);
}

/// Reflexive trait for `System`
/// every container that is used by a system implements this
pub trait RunSystem<In>: Container<In> {
    fn run<S>(&mut self, system: &mut S)
    where
        S: System<In, Collection = Self>,
    {
        system.run_system(self);
    }
}
impl<C, In> RunSystem<In> for C where C: Container<In> {}
