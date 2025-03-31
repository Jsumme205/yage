/// macro that defines a schema for component data
/// this takes in a struct that looks soemthing like this:
///
/// ```
/// pub struct Enemy<T> {
///     id -> i32,
///     weapon -> T
/// }
/// ```
///
/// and output a lot of structures to manage it.
///
/// the core struct it will output would be the `Enemy` struct, which would like this:
///
/// ```
/// #[repr(C)]
/// pub struct Enemy<T> {
///     header: Header,
///     entities: ThinSlice<Entity>,
///     id: ThinSlice<i32>,
///     weapon: ThinSlice<T>
/// }
/// ```
///
/// this logically represents something sort of like a `Vec<Enemy<T>>`
/// but is more cache-friendly and allows the compiler to make some optimizations including vectorization
#[macro_export]
macro_rules! component_data {
    (
        $vis:vis struct $Name:ident  $(<$($gen:tt),*>)? {
            $($field_name:ident -> $FieldTy:ty),* $(,)?
        }

    ) => {
        // definition of core structs
        ::paste::paste! {

            $vis struct $Name $(<$($gen),*>)? {
                header: $crate::Header<[<$Name Layout>]>,
                entities: $crate::ThinSlice<::core::sync::atomic::AtomicU64>,
                $($field_name: $crate::ThinSlice<$FieldTy>),*
            }

            $vis struct [<$Name Instance>] $(<$($gen),*>)? {
                $($field_name: $FieldTy),*
            }
        }

        // layout information
        ::paste::paste! {
            #[derive(Clone, Copy, Debug)]
            struct [<$Name Layout>] {
                size: usize,
                align: usize,
                $([<$field_name _offset>]: usize),*
            }

            impl [<$Name Layout>] {

                pub const fn [<eval_ $Name:snake _layout>](num: u32) -> Self {
                    let number = num as usize;
                    let align = ::core::mem::align_of::<Self>();
                    let entity_size = $crate::component_data!(@@@size_for: ::core::sync::atomic::AtomicU64, number, align);
                    $(
                        let [<$field_name _size>] = $crate::component_data!(@@@size_for: $FieldTy, number, align);
                    )*

                    let mut layout = unsafe { ::core::mem::zeroed::<Self>() };
                    let mut offset = entity_size;
                    layout.align = align;
                    $(
                        $crate::component_data!(@@@write_layout: layout.[<$field_name _offset>], [<$field_name _size>], offset);
                    )*
                    layout.size = offset;
                    layout
                }
            }

            impl $crate::GetLayout for [<$Name Layout>] {
                fn layout(&self) -> ::core::alloc::Layout {
                    unsafe {::core::alloc::Layout::from_size_align_unchecked(self.size, self.align)}
                }
            }
        }

        // allocation implementation
        ::paste::paste! {
            impl $(<$($gen),*>)? $Name $(<$($gen),*>)?{

                $vis fn allocate<A>(number: u32, allocator: &mut A) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator

                {
                    let layout: &'static _ =
                        Box::leak(Box::new([<$Name Layout>]::[<eval_ $Name:snake _layout>](number)));
                    let header = $crate::Header::new(false, number, layout);
                    Self::__allocate_inner(allocator, header)
                }

                $vis fn allocate_array<A, const N: usize>(allocator: &mut A) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator
                {
                    let header = $crate::Header::new(
                        true,
                        N as _,
                        const {&[<$Name Layout>]::[<eval_ $Name:snake _layout>](N as _)}
                    );
                    Self::__allocate_inner(allocator, header)
                }

                fn __allocate_inner<A>(
                    allocator: &mut A,
                    header: $crate::Header<[<$Name Layout>]>
                ) -> Result<Self, A::Error>
                where
                    A: ::yage_core::allocator_api::Allocator
                {
                    use yage_core::allocator_api::Allocator;
                    let ptr = allocator.allocate(header.layout())?;
                    let raw = ptr.as_ptr();
                    let layout = header.layout;
                    let this = unsafe {
                        Self {
                            header,
                            entities: $crate::ThinSlice::from_raw(raw.cast()),
                            $($field_name: $crate::component_data!(@@@thin_slice: raw, layout.[<$field_name _offset>])),*
                        }
                    };
                    Ok(this)
                }
            }

            impl $(<$($gen),*>)? Drop for $Name $(<$($gen),*>)? {

                fn drop(&mut self) {


                }

            }

            //$vis const fn push()
        }

        ::paste::paste! {
            #[derive(Clone, Copy)]
            $vis struct [<$Name Ref>]<'__ref, $($($gen),*)?> {
                $($field_name: &'__ref $FieldTy),*
            }

            $vis struct [<$Name RefMut>]<'__ref $($($gen),*)?> {
                $($field_name: &'__ref mut $FieldTy),*
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
    };
    (@@@unwrap_result: $expr:expr) => {
        match $expr {
            Ok(r) => r,
            Err(_) => panic!("error")
        }
    };
    (@@@size_for: $T:ty, $num:expr, $align:expr) => {
        {
            let layout = $crate::component_data!(@@@unwrap_result: ::core::alloc::Layout::array::<$T>($num));
            let layout = $crate::component_data!(@@@unwrap_result: layout.align_to($align));
            layout.size()
        }
    };
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
