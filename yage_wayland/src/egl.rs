use libloading::{Lib, Library, Symbol};
use spin::lazy::Lazy;
use std::{
    ffi::{c_void, CString},
    sync::{LazyLock, OnceLock},
};

#[allow(dead_code)]
struct EglLib;

type EglGetProcAddr = unsafe extern "system" fn(*const c_void) -> *const c_void;
static ADDR_LOADER: OnceLock<Symbol<EglGetProcAddr>> = OnceLock::new();

impl Lib for EglLib {
    const NAMES: &'static [&'static str] = &[
        "/usr/lib/libEGL.so.1",
        "/usr/lib/libEGL.so",
        "/usr/lib64/libEGL.so",
        "/usr/lib64/libEGL.so.1",
    ];
}

static EGL: LazyLock<Library<EglLib>> = LazyLock::new(|| {
    let lib = Library::load_static().unwrap();
    lib
});

static EGL_API: Lazy<Option<EglApi>> = Lazy::new(|| {
    let loader = move |sym: &'static str| -> *const c_void {
        unsafe {
            if let Ok(sym) = EGL.load_sym_raw(sym) {
                // SAFETY: we can do this because it's just a thin wrapper around a c_void
                return sym.as_ptr();
            }

            let egl_proc_address = ADDR_LOADER.get_or_init(|| {
                let sym: Symbol<EglGetProcAddr> = EGL.load_sym(b"eglGetProcAddress\0").unwrap();
                sym
            });

            let sym = CString::new(sym.as_bytes()).unwrap();

            egl_proc_address.call(sym.as_bytes_with_nul().as_ptr() as *const _)
        }
    };

    macro_rules! sym {
        ($s:literal) => {
            Symbol::from_raw(loader($s))
        };
    }

    let egl = unsafe {
        EglApi {
            BindApi: sym!("eglBindApi"),
            BindTexImage: sym!("eglBindTexImage"),
            ChooseConfig: sym!("eglChooseConfig"),
            CreateContext: sym!("eglChooseConfig"),
            GetConfigs: sym!("eglGetConfigs"),
            GetError: sym!("eglGetError"),
            GetDisplay: sym!("eglGetDisplay"),
        }
    };

    Some(egl)
});

#[allow(non_snake_case)]
pub(crate) struct EglApi {
    pub(crate) BindApi: Symbol<unsafe extern "C" fn(u32) -> bool>,
    pub(crate) BindTexImage: Symbol<unsafe extern "C" fn(*mut c_void, *mut c_void, i32) -> u32>,
    pub(crate) ChooseConfig: Symbol<
        unsafe extern "C" fn(*mut c_void, *const i32, *mut *mut c_void, i32, *mut i32) -> u32,
    >,
    pub(crate) CreateContext: Symbol<unsafe extern "C" fn()>,
    pub(crate) GetConfigs:
        Symbol<unsafe extern "C" fn(*mut c_void, *mut *mut c_void, i32, *mut i32) -> u32>,
    pub(crate) GetError: Symbol<unsafe extern "C" fn() -> i32>,
    pub(crate) GetDisplay: Symbol<unsafe extern "C" fn(*mut c_void) -> *mut c_void>,
}
