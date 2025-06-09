use std::{
    f32::consts::E,
    ffi::{CStr, CString, OsStr, c_char, c_int, c_void},
    io,
    marker::PhantomData,
    path::Path,
    ptr::NonNull,
};

pub mod fn_ptr;

pub use fn_ptr::{FnPtr, Symbol};

#[link(name = "ld")]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;

    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;

    fn dlerr() -> *mut c_char;
}

pub trait Lib {
    const NAMES: &'static [&'static str];
}

pub struct Library<L> {
    handle: NonNull<c_void>,
    _marker: PhantomData<L>,
}

unsafe impl<L> Send for Library<L> where L: Lib {}
unsafe impl<L> Sync for Library<L> where L: Lib {}

impl<L> Library<L>
where
    L: Lib,
{
    pub fn load_static() -> io::Result<Self> {
        for name in L::NAMES {
            let lib = Library::load_dynamically(*name);
            if let Ok(lib) = lib {
                return unsafe { Ok(std::mem::transmute(lib)) };
            }
        }
        Err(get_error())
        // SAFETY: this carries the same size, we're just transmuting the marker basically
    }
}

fn get_error() -> io::Error {
    let error = unsafe { dlerr() };
    if !error.is_null() {
        let str: CString = unsafe { CStr::from_ptr(error).into() };
        return io::Error::new(io::ErrorKind::Other, str.to_string_lossy());
    }
    io::Error::other("cannnot find error")
}

impl Library<()> {
    pub fn load_dynamically<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let filename = path.as_ref();
        let p = filename.as_os_str().as_encoded_bytes().as_ptr();
        let handle = unsafe {
            match NonNull::new(dlopen(p.cast(), 1 | 0)) {
                Some(handle) => Ok(Self {
                    handle,
                    _marker: PhantomData,
                }),
                None => {
                    let error = dlerr();
                    if !error.is_null() {
                        let str: CString = CStr::from_ptr(error).into();
                        return Err(io::Error::new(io::ErrorKind::Other, str.to_string_lossy()));
                    }
                    Err(io::Error::other("cannnot find error"))
                }
            }
        };
        handle
    }
}

impl<L> Library<L> {
    pub fn load_sym<F>(&self, name: impl AsRef<[u8]>) -> io::Result<Symbol<F>>
    where
        F: FnPtr,
    {
        let name = CString::new(name.as_ref()).unwrap();
        let symbol = unsafe {
            dlsym(
                self.handle.as_ptr(),
                name.as_bytes_with_nul().as_ptr().cast(),
            )
        };
        if symbol.is_null() {
            return Err(get_error());
        } else {
            Ok(unsafe { Symbol::from_raw_handle(symbol) })
        }
    }

    pub unsafe fn load_sym_raw(&self, name: impl AsRef<[u8]>) -> io::Result<NonNull<c_void>> {
        let name = CString::new(name.as_ref()).unwrap();
        let symbol = unsafe {
            dlsym(
                self.handle.as_ptr(),
                name.as_bytes_with_nul().as_ptr().cast(),
            )
        };
        if symbol.is_null() {
            return Err(get_error());
        } else {
            Ok(unsafe { NonNull::new_unchecked(symbol) })
        }
    }
}

impl<L> Drop for Library<L> {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.handle.as_ptr());
        }
    }
}
