use std::{f64::consts::PI, ffi::c_void, marker::PhantomData, ptr::NonNull};

mod __sealed {
    pub trait Sealed {}

    macro_rules! impl_sealed {
        () => {};
        ($first:tt $(, $rest:tt)*) => {
            impl<R, $first, $($rest),*> Sealed for unsafe extern "C" fn($first, $($rest),*) -> R {}
            impl_sealed!($($rest),*);
        };
    }

    impl<R, T> Sealed for unsafe extern "system" fn(T) -> R {}
    impl<R> Sealed for unsafe extern "C" fn() -> R {}
    impl_sealed!(T1, T2, T3, T4, T5, T6, T7, T8);
}

macro_rules! impl_fnptr {
    () => {};
    ($first:tt $(, $rest:tt)*) => {
        unsafe impl<R, $first, $($rest),*> self::FnPtr for unsafe extern "C" fn($first, $($rest),*) -> R {
            type Output = R;
            type Args = ($first, $($rest),*);

            unsafe fn from_raw_unchecked(ptr: *mut ()) -> Self {
                unsafe { core::mem::transmute(ptr) }
            }

            fn into_raw(self) -> *mut () {
                self as *mut ()
            }

            #[allow(non_snake_case, unused_unsafe)]
            unsafe fn call(self, args: Self::Args) -> R {
                let ($first, $($rest),*) = args;
                unsafe {
                    (self)($first, $($rest),*)
                }
            }
        }

        impl_fnptr!($($rest),*);
    }
}

impl_fnptr!(T1, T2, T3, T4, T5, T6, T7, T8);

pub unsafe trait FnPtr: Sized + Copy + __sealed::Sealed {
    type Output;
    type Args;

    unsafe fn from_raw_unchecked(ptr: *mut ()) -> Self;

    unsafe fn from_raw(ptr: *mut ()) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Self::from_raw_unchecked(ptr) })
        }
    }

    fn into_raw(self) -> *mut ();

    unsafe fn call(self, args: Self::Args) -> Self::Output;
}

unsafe impl<R, T> FnPtr for unsafe extern "system" fn(T) -> R {
    type Args = T;
    type Output = R;

    unsafe fn from_raw_unchecked(ptr: *mut ()) -> Self {
        unsafe { core::mem::transmute(ptr) }
    }

    unsafe fn call(self, args: Self::Args) -> Self::Output {
        unsafe { (self)(args) }
    }

    fn into_raw(self) -> *mut () {
        self as *mut _
    }
}

unsafe impl<R> FnPtr for unsafe extern "C" fn() -> R {
    type Args = ();
    type Output = R;

    unsafe fn from_raw_unchecked(ptr: *mut ()) -> Self {
        unsafe { std::mem::transmute(ptr) }
    }

    fn into_raw(self) -> *mut () {
        self as *mut _
    }

    unsafe fn call(self, _: Self::Args) -> Self::Output {
        unsafe { (self)() }
    }
}

pub struct Symbol<F> {
    handle: NonNull<c_void>,
    _marker: PhantomData<F>,
}

impl<F> Symbol<F>
where
    F: FnPtr,
{
    pub(crate) unsafe fn from_raw_handle(handle: *mut c_void) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
            _marker: PhantomData,
        }
    }

    pub unsafe fn call(&self, args: F::Args) -> F::Output {
        let fun = unsafe { F::from_raw_unchecked(self.handle.as_ptr() as *mut ()) };
        unsafe { fun.call(args) }
    }

    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        unsafe {
            Self {
                handle: NonNull::new_unchecked(ptr as *mut _),
                _marker: PhantomData,
            }
        }
    }
}

unsafe impl<F: Send> Send for Symbol<F> {}
unsafe impl<F: Sync> Sync for Symbol<F> {}
