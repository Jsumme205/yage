use core::convert::Infallible;
use yage_core::allocator_api::Allocator;
use yage_util::container_trait::Container;

mod example;
pub mod utility_structs;

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

/// field layout:
/// entities
/// ========
/// $field_name 0
/// ========
/// $field_name 1
/// ========
/// ...
/// ========
/// $field_name N

macro_rules! component_data {
    (
        $vis:vis struct $Name:ident  $(<$($gen:tt),*>)? {
            $($field_name:ident -> $FieldTy:ty),* $(,)?
        }

    ) => {
        //$vis struct $name $(<$($gen),*>)?

        ::paste::paste! {
            #[repr(C)]
            $vis struct $Name $(<$($gen),*>)? {
                header: $crate::utility_structs::Header<[<$Name Layout>]>,
                entities: $crate::utility_structs::ThinSlice<$crate::utility_structs::Entity>,
                $($field_name: $crate::utility_structs::ThinSlice<$FieldTy>),*
            }

            $vis struct [<$Name Instance>] $(<$($gen),*>)? {
                $($field_name: $FieldTy),*
            }

            $vis struct [<$Name Ref>] <'__ref, $(($gen),*)?> {
                $($field_name: &'__ref $FieldTy),*
            }

            $vis struct [<$Name Mut>] <'__ref, $(($gen),*)?> {
                $($field_name: &'__ref mut $FieldTy),*
            }

            component_data!(@@@impl_represents: &'a [<$Name Instance>], [<$Name Ref>]<'a, $($($gen),*)?>);
            component_data!(@@@impl_represents: &'a mut [<$Name Instance>], [<$Name Mut>]<'a, $($($gen),*)?>);

            #[derive(Copy, Clone)]
            struct [<$Name Layout>] {
                align: usize,
                size: usize,
                $([<$field_name _offset>]: usize),*,
            }

            impl $crate::utility_structs::GetLayout for [<$Name Layout>] {
                fn layout(&self) -> ::core::alloc::Layout {
                       unsafe { ::core::alloc::Layout::from_size_align_unchecked(self.size, self.align) }
                }
            }

            $vis struct [<$Name Iter>] <'__ref, $($($gen),*)?> {
                inner: &'__ref $Name $(<$($gen),*>)?,
                idx: u32,
                _marker: ::core::marker::PhantomData<&'__ref [[<$Name Instance>]]>
            }

            impl<'__ref, $($($gen),*)?> Iterator for [<$Name Iter>]<'__ref, $($($gen),*)?> {
                type Item = [<$Name Ref>]<'__ref, $($($gen),*)?>;


                fn next(&mut self) -> Option<Self::Item> {
                    todo!()
                }
            }

            impl<'__ref, $($($gen),*)?> Iterator for [<$Name IterMut>]<'__ref, $($($gen),*)?> {
                type Item = [<$Name Mut>]<'__ref, $($($gen),*)?>;

                fn next(&mut self) -> Option<Self::Item> {
                    todo!()
                }
            }

            $vis struct [<$Name IterMut>] <'__ref, $($($gen),*)?> {
                inner: &'__ref mut $Name $(<$($gen),*>)?,
                idx: u32,
                _marker: ::core::marker::PhantomData<&'__ref mut [[<$Name Instance>]]>
            }
        }


        // the core implementation, does all the heavy lifting
        ::paste::paste! {

            impl [<$Name Layout>] {

                const fn size_padded_to_align<T>(number: usize) -> usize {
                    const ALIGN: usize = ::core::mem::align_of::<[<$Name Instance>]>();

                    unsafe {
                        ::core::alloc::Layout::from_size_align_unchecked(::core::mem::size_of::<T>() * number, ALIGN)
                        .pad_to_align()
                        .size()
                    }
                }

                const fn eval_this_layout(number: u32) -> Self {
                    let number = number as usize;
                    let align = ::core::mem::align_of::<[<$Name Instance>]>();
                    let entity_size = Self::size_padded_to_align::<$crate::utility_structs::Entity>(number);
                    $(
                        let [<$field_name _size>] = Self::size_padded_to_align::<$FieldTy>(number);
                    )*

                    let mut layout = unsafe { ::core::mem::zeroed::<[<$Name Layout>]>() };
                    let mut offset = entity_size;
                    layout.align = align;

                    $(
                        component_data!(@@@write_layout: layout.[<$field_name _offset>], [<$field_name _size>], offset);
                    )*

                    layout.size = offset;
                    layout
                }


            }

            impl $(<$($gen),*>)? $Name $(<$($gen),*>)? {

                $vis fn allocate<A>(number: u32, allocator: &mut A) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator
                {

                    let layout: &'static [<$Name Layout>] =
                        ::std::boxed::Box::leak(
                            ::std::boxed::Box::new(
                                [<$Name Layout>]::eval_this_layout(number)
                            )
                        );
                    let header = $crate::utility_structs::Header::new(false, number, layout);
                    Self::__allocate_inner(allocator, header)
                }

                $vis fn allocate_array_layout<A, const N: usize>(allocator: &mut A) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator
                {
                    let header = $crate::utility_structs::Header::new(
                        true,
                        N as _,
                        const {&[<$Name Layout>]::eval_this_layout(N as _)}
                    );
                    Self::__allocate_inner(allocator, header)
                }

                fn __allocate_inner<A>(
                    allocator: &mut A,
                    header: $crate::utility_structs::Header<[<$Name Layout>]>
                ) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator,
                {

                    let ptr = allocator.allocate(header.layout())?;
                    let raw = ptr.as_ptr();
                    let layout = header.layout;
                    let this = unsafe {
                        Self {
                            header,
                            entities: $crate::utility_structs::ThinSlice::from_raw(raw.cast()),
                            $($field_name: component_data!(@@@thin_slice: raw, layout.[<$field_name _offset>])),*
                        }
                    };
                    Ok(this)
                }

                $vis const fn push(&mut self, item: [<$Name Instance>]) -> ::yage_core::Result<$crate::utility_structs::Entity> {
                    if self.header.is_full() {
                        return Err(::yage_core::errors::Error::new(
                            ::yage_core::ErrorKind::PushError
                        ));
                    }

                    let [<$Name Instance>] {
                        $($field_name),*
                    } = item;

                    unsafe {
                        let len = self.header.len() as usize;
                        let align = self.header.raw_layout().align;
                        $(
                            component_data!(@@@write_ptr: self.$field_name.as_mut_ptr(), len, align, $field_name);
                        )*

                        self.header.set_len((len + 1) as _);
                    }
                    Ok($crate::utility_structs::Entity::DUMMY)
                }
            }

            impl $(<$($gen),*>)? ::yage_util::container_trait::Container<[<$Name Instance>]> for $Name {
                type Iterator<'a> = [<$Name Iter>]<'a, $($($gen),*)?>;
                type Mutable<'a> = [<$Name IterMut>]<'a, $($($gen),*)?>;

                fn iterator(&self) -> Self::Iterator<'_> {
                    [<$Name Iter>] {
                        inner: self,
                        idx: 0,
                        _marker: Default::default(),
                    }
                }

                fn mutable_iterator(&mut self) -> Self::Mutable<'_> {
                    [<$Name IterMut>] {
                        inner: self,
                        idx: 0,
                        _marker: Default::default(),
                    }
                }
            }
        }
    };

    // internal macros, do not use!
    (@@@write_layout: $place:expr, $val:expr, $off:expr) => {
        $place = $off;
        $off += $val;
    };
    (@@@impl_represents: $ty:ty, $impl_ref:ty) => {
        impl<'a> ::yage_util::container_trait::Represents<$ty> for $impl_ref {}
    };
    (@@@write_ptr: $place:expr, $len:expr, $align:expr, $val:expr) => {{
        let ptr = $place.byte_add($len * $align);
        ptr.write_unaligned($val);
    }};
    (@@@thin_slice: $raw:expr, $offset:expr) => {
        $crate::utility_structs::ThinSlice::from_raw($raw.add($offset).cast())
    }

}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use super::*;

    component_data! {
        pub struct Player {
            name -> &'static str,
            id -> i32,
        }
    }

    #[test]
    fn test_alloc_worked() {
        //let player = Player::allocate(2, &mut StdAlloc).unwrap();
    }
}
