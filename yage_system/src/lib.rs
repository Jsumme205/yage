use core::convert::Infallible;
use yage_core::allocator_api::Allocator;
use yage_util::container_trait::Container;

mod example;
pub mod utility_structs;

/// the core trait for systems
/// systems can be thought of as a function that takes an input `In`, and does an operation on it
pub trait System<In> {
    type Collection: Container<In>;

    fn run_system(&mut self, collection: &mut Self::Collection);
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
        $vis:vis struct $name:ident  $(<$($gen:tt),*>)? {
            $($field_name:ident :  $field_ty:ty),* $(,)?
        }

    ) => {
        $vis struct $name $(<$($gen),*>)? {

            header: $crate::utility_structs::Header,
            entities: *mut $crate::utility_structs::Entity,
            $($field_name: *mut $field_ty),*,
            _pin: core::marker::PhantomPinned
        }

        ::paste::paste! {
            #[derive(Clone, Copy, Debug)]
            struct [<$name Layout>] {
                $([<$field_name _offset>]: usize),*
            }
        }

        ::paste::paste! {
            #[derive(Clone, Copy)]
            $vis struct [<$name Ref>] <'a, $(($gen),*)?> {
                entity: &'a $crate::utility_structs::Entity,
                $($field_name: &'a $field_ty),*
            }
        }


        impl $(<$($gen),*>)?  $name $(<$($gen),*>)? {

            #[inline(always)]
            pub(crate) const fn largest_align() -> usize {
                ::core::mem::align_of::<Self>()
            }

            ::paste::paste! {
                fn eval_this_layout(number: u32) -> ([<$name Layout>], usize) {
                    let mut total_offset = 0;
                    let mut layout: [<$name Layout>] = unsafe {::core::mem::zeroed()};
                    $(
                        let offset = core::mem::size_of::<$field_ty>() * (number as usize);
                        layout.[<$field_name _offset>] = total_offset;
                        total_offset += offset;
                    )*
                    (layout, total_offset)
                }
            }

            pub fn allocate<A>(number: u32, allocator: &mut A) -> Result<Self, A::Error>
            where
                A: ::yage_core::allocator_api::Allocator
            {
                let (layout, actual_mem_size) = std::dbg!(Self::eval_this_layout(number));
                //let _ptr = //::yage_core::allocator_api::Allocator::allocate(
                    //allocator,
                    //unsafe {::std::alloc::Layout::from_size_align_unchecked(Self::largest_align(), actual_mem_size)}
               // )?;

                Ok(Self::dangling())
            }

            pub(crate) const fn dangling() -> Self {
                Self {
                    header: $crate::utility_structs::Header::DANGLING,
                    entities: core::ptr::null_mut(),
                    $($field_name: core::ptr::null_mut()),*,
                    _pin: core::marker::PhantomPinned,
                }
            }
        }



    };
}

component_data! {
    pub struct Player {
        id: u32,
        name: &'static str,
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::NonNull;

    use super::*;

    #[test]
    fn test_correct_align() {
        assert!(Player::largest_align() == 0x8);
        assert!(Player::largest_align().is_power_of_two())
    }

    #[test]
    fn test_alloc_worked() {
        //let player = Player::allocate(2, &mut StdAlloc).unwrap();
    }
}
