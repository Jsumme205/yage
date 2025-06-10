use core::ffi;
use std::collections::HashSet;
use std::ffi::CStr;
use std::fmt;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};

use std::os::unix::net::UnixStream;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use bindings::wl_argument;
use data::{DummyObjectData, ObjectData};

/// internal stuff
mod bindings;
pub mod enums;
pub mod interfaces;
pub mod protocol_structs;

/// exported modules, user-public API's
pub mod data;
pub mod event;
pub mod event_loop;
pub mod protocol;

use event::{Arg, Event};
use event_loop::{EventQueue, EventQueueInner, RUST_MANAGED};
use protocol::Protocol;

pub trait OptionExt<T> {
    fn into_ptr(self) -> *mut T;
}

impl OptionExt<bindings::wl_proxy> for Option<NonNull<bindings::wl_proxy>> {
    fn into_ptr(self) -> *mut bindings::wl_proxy {
        unsafe { core::mem::transmute(self) }
    }
}

//scoped_tls::scoped_thread_local!(pub(crate) static BACKEND: (Wayland, EventQueue));

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ArgKind {
    Int,
    Uint,
    Fixed,
    Str(bool),
    Object(bool),
    New,
    Array,
    Fd,
}

struct Inner {
    pub(crate) display: NonNull<bindings::wl_display>,
    pub(crate) proxy: Proxy,
}

unsafe impl Sync for Inner {}

impl Inner {
    pub(crate) fn connect(stream: UnixStream) -> crate::Result<Self> {
        let display = unsafe {
            match NonNull::new(bindings::wl_display_connect_to_fd(stream.into_raw_fd())) {
                Some(p) => p,
                None => {
                    return Err(crate::Error::new(crate::ErrorKind::AllocError(
                        "wl_display",
                    )))
                }
            }
        };

        Ok(Self {
            display,
            proxy: unsafe {
                Proxy::new_(
                    display.cast(),
                    Option::None,
                    &interfaces::WL_DISPLAY_INTERFACE,
                    1,
                )
            },
        })
    }

    pub(crate) fn send_message_inner(
        &mut self,
        message: Event<ObjectId, RawFd>,
        data: Option<Arc<dyn ObjectData>>,
        child_spec: Option<(&'static Interface, u32)>,
        mut eq: MutexGuard<'_, EventQueueInner>,
    ) -> crate::Result<ObjectId> {
        let Event {
            sender,
            opcode,
            args,
        } = message;

        let message_d = match sender.iface.requests.get(message.opcode as usize) {
            Some(msg) => msg,
            None => return Err(crate::Error::new(crate::ErrorKind::Other)),
        };

        if !sender
            .alive
            .as_ref()
            .map(|a| a.load(Ordering::Acquire))
            .unwrap_or(true)
            || sender.proxy.is_none()
        {
            return Err(crate::Error::new(crate::ErrorKind::InvalidId));
        }

        let parent_version = if sender.id == 1 {
            1
        } else {
            unsafe { bindings::wl_proxy_get_version(sender.proxy.unwrap().as_ptr()) }
        };

        // if !check_for_sig () {}

        let child_spec = if message_d.sig.iter().any(|arg| matches!(arg, ArgKind::New)) {
            if let Some((iface, version)) = child_spec {
                if let Some(child_iface) = message_d.child {
                    if !same_interface(child_iface, iface) {
                        return Err(crate::Error::new(crate::ErrorKind::InvalidInterface));
                    }
                    if version != parent_version {
                        return Err(crate::Error::new(crate::ErrorKind::InvalidId));
                    }
                }
                Some((iface, version))
            } else if let Some(child_iface) = message_d.child {
                Some((child_iface, parent_version))
            } else {
                return Err(crate::Error::new(crate::ErrorKind::InvalidId));
            }
        } else {
            None
        };

        let child_iface_ptr = child_spec
            .as_ref()
            .map(|(i, _)| i._raw_interface.expect("this shouldn't be null").as_ptr() as *const _)
            .unwrap_or(core::ptr::null());
        let child_version = child_spec
            .as_ref()
            .map(|(_, v)| *v)
            .unwrap_or(parent_version);

        let mut arg_list = Vec::with_capacity(args.len());
        let mut arg_interfaces = message_d.arg_interfaces.iter();
        for (i, arg) in args.into_iter().enumerate() {
            use Arg::*;
            match arg {
                Uint(u) => arg_list.push(bindings::wl_argument { u }),
                Int(i) => arg_list.push(bindings::wl_argument { i }),
                Fixed(f) => arg_list.push(bindings::wl_argument { f }),
                Fd(h) => arg_list.push(bindings::wl_argument { h }),
                Array(bytes) => {
                    let len = bytes.len();
                    let ptr = Box::into_raw(bytes);
                    let a = Box::new(bindings::wl_array {
                        data: ptr as *mut u8 as *mut _,
                        alloc: len,
                        size: len,
                    });
                    arg_list.push(bindings::wl_argument {
                        a: Box::into_raw(a),
                    });
                }
                Str(Some(s)) => {
                    arg_list.push(bindings::wl_argument {
                        s: s.into_c_string().into_raw(),
                    });
                }
                Str(None) => arg_list.push(bindings::wl_argument {
                    s: core::ptr::null(),
                }),
                Object(o) => {
                    let next_interface = arg_interfaces.next().unwrap();
                    if !o.proxy.is_none() {
                        if !o
                            .alive
                            .as_ref()
                            .map(|a| a.load(Ordering::Acquire))
                            .unwrap_or(true)
                        {
                            unsafe { free_arrays(message_d.sig, &mut *arg_list) }
                            return Err(crate::ErrorKind::SubmitError.into_error());
                        }
                        if !same_interface(next_interface, o.iface) {
                            panic!("invalid interface")
                        }
                    } else if !matches!(message_d.sig[i], ArgKind::Object(true),) {
                        panic!(
                            "Request {}@{}.{} expects an non-null object argument.",
                            sender.iface.name, sender.id, message_d.name
                        );
                    }
                    arg_list.push(bindings::wl_argument {
                        o: o.proxy.unwrap().as_ptr() as *mut _,
                    });
                }
                New(_) => arg_list.push(bindings::wl_argument { n: 0 }),
            }
        }

        let ret = if child_spec.is_none() {
            unsafe {
                bindings::wl_proxy_marshal_array(
                    core::mem::transmute(sender.proxy),
                    opcode as u32,
                    arg_list.as_mut_ptr(),
                );
                None
            }
        } else {
            let wrapped =
                unsafe { bindings::wl_proxy_create_wrapper(core::mem::transmute(sender.proxy)) };
            let ret = unsafe {
                bindings::wl_proxy_marshal_array_constructor_versioned(
                    wrapped as *mut _,
                    opcode as u32,
                    arg_list.as_mut_ptr(),
                    child_iface_ptr,
                    child_version,
                )
            };
            unsafe {
                bindings::wl_proxy_wrapper_destroy(wrapped);
            }
            NonNull::new(ret)
        };

        unsafe {
            free_arrays(message_d.sig, &mut *arg_list);
        }

        if ret.is_none() && child_spec.is_some() {
            return Err(crate::ErrorKind::AllocError("wl_proxy_marshal_array").into_error());
        }

        let child_id = if let Some((child_iface, _)) = child_spec {
            let data = match data {
                Some(data) => data,
                None => unsafe {
                    bindings::wl_proxy_destroy(ret.into_ptr());
                    panic!("sending a request w/o user data")
                },
            };
            unsafe { eq.__manage_object(child_iface, ret.into_ptr(), data) }
        } else {
            ObjectId::NIL_OBJECT_ID
        };

        if message_d.is_destructor {
            if let Some(ref alive) = sender.alive {
                let udata = unsafe {
                    let p =
                        bindings::wl_proxy_get_user_data(sender.proxy.into_ptr()) as *mut ProxyData;
                    Box::from_raw(p)
                };
                unsafe {
                    bindings::wl_proxy_set_user_data(
                        sender.proxy.into_ptr(),
                        core::ptr::null_mut(),
                    );
                }
                alive.store(false, Ordering::Release);
                udata.data.unwrap().on_destruction(sender.clone());
            }
            eq.known_proxies.remove(&sender.proxy.into_ptr());

            unsafe {
                bindings::wl_proxy_destroy(sender.proxy.into_ptr());
            }
        }
        Ok(child_id)
    }

    pub(crate) fn queue(&mut self) -> crate::Result<EventQueue> {
        let queue = unsafe { bindings::wl_display_create_queue(self.display.as_ptr()) };
        if queue.is_null() {
            return Err(crate::Error::new(crate::ErrorKind::Other));
        } else {
            Ok(unsafe { EventQueue::new(queue) })
        }
    }

    pub(crate) fn inner_queue(&mut self) -> crate::Result<EventQueueInner> {
        let queue = unsafe { bindings::wl_display_create_queue(self.display.as_ptr()) };
        if queue.is_null() {
            return Err(crate::Error::new(crate::ErrorKind::Other));
        } else {
            Ok(unsafe {
                EventQueueInner {
                    eq: NonNull::new_unchecked(queue),
                    known_proxies: HashSet::new(),
                }
            })
        }
    }

    pub(crate) fn flush(&mut self) -> crate::Result<()> {
        let ret = unsafe { bindings::wl_display_flush(self.display.as_ptr()) };
        if ret < 0 {
            Err(crate::Error::new(crate::ErrorKind::FlushError))
        } else {
            Ok(())
        }
    }

    pub(crate) fn fd(&mut self) -> BorrowedFd {
        unsafe { BorrowedFd::borrow_raw(bindings::wl_display_get_fd(self.display.as_ptr())) }
    }
}

pub struct Proxy {
    data: ProxyData,
    id: ObjectId,
}

#[repr(C)]
pub struct ProxyData {
    pub alive: Arc<AtomicBool>,
    pub data: Option<Arc<dyn ObjectData>>,
    pub interface: &'static Interface,
}

#[derive(Clone, Debug)]
pub struct ObjectId {
    pub proxy: Option<NonNull<bindings::wl_proxy>>,
    pub id: u32,
    pub iface: &'static Interface,
    pub alive: Option<Arc<AtomicBool>>,
}

pub struct ObjectMetadata {
    pub id: ObjectId,
    pub iface: &'static Interface,
    pub version: u32,
}

impl ObjectId {
    pub(crate) const NIL_OBJECT_ID: ObjectId = ObjectId {
        proxy: None,
        id: 0,
        iface: &Interface::ANON,
        alive: None,
    };

    pub fn is_null(&self) -> bool {
        self.alive.is_none()
            && self.proxy.is_none()
            && self.id == 0
            && same_interface(self.iface, &Interface::ANON)
    }
}

impl Proxy {
    unsafe fn new_(
        proxy: NonNull<()>,
        data: Option<Arc<dyn ObjectData>>,
        interface: &'static Interface,
        id: u32,
    ) -> Self {
        let alive = Arc::new(AtomicBool::new(true));

        Self {
            data: ProxyData {
                alive: Arc::clone(&alive),
                data,
                interface,
            },
            id: ObjectId {
                proxy: Some(proxy.cast()),
                alive: Some(alive),
                id,
                iface: interface,
            },
        }
    }
}

pub struct Interface {
    pub name: &'static str,
    pub version: u32,
    pub requests: &'static [Message],
    pub events: &'static [Message],
    pub _raw_interface: Option<NonNull<bindings::wl_interface>>,
}

impl fmt::Debug for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interface")
            .field("version", &self.version)
            .field("requests", &self.requests)
            .field("events", &self.events)
            .finish_non_exhaustive()
    }
}

pub fn same_interface(i1: &'static Interface, i2: &'static Interface) -> bool {
    std::ptr::eq(i1, i2) || i1.name == i2.name
}

unsafe impl Send for Interface {}
unsafe impl Sync for Interface {}

impl Interface {
    pub(crate) const ANON: Self = Self {
        name: "<anonymous>",
        version: 0,
        requests: &[],
        events: &[],
        _raw_interface: None,
    };

    pub const fn new(name: &'static str, version: u32) -> Self {
        Self {
            name,
            version,
            requests: &[],
            events: &[],
            _raw_interface: None,
        }
    }

    pub const fn requests(mut self, requests: &'static [Message]) -> Self {
        self.requests = requests;
        self
    }

    pub const fn events(mut self, events: &'static [Message]) -> Self {
        self.events = events;
        self
    }

    pub const fn iface(mut self, iface: NonNull<bindings::wl_interface>) -> Self {
        self._raw_interface = Some(iface);
        self
    }

    pub const fn interface(mut self, iface: &'static bindings::wl_interface) -> Self {
        self._raw_interface =
            Some(unsafe { NonNull::new_unchecked((&raw const *iface) as *mut _) });
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub name: &'static str,
    pub sig: &'static [ArgKind],
    pub since: u32,
    pub is_destructor: bool,
    pub child: Option<&'static Interface>,
    pub arg_interfaces: &'static [&'static Interface],
}

// common messages
impl Message {
    pub const DESTROY: Self = Self {
        name: "destroy",
        sig: &[],
        since: 1,
        is_destructor: true,
        child: None,
        arg_interfaces: &[],
    };

    pub const fn new(name: &'static str) -> Self {
        Self::with_sig(name, &[])
    }

    pub const fn with_sig(name: &'static str, sig: &'static [ArgKind]) -> Self {
        Self {
            name,
            sig,
            since: 1,
            is_destructor: false,
            child: None,
            arg_interfaces: &[],
        }
    }

    pub const fn since(mut self, since: u32) -> Self {
        self.since = since;
        self
    }

    pub const fn child(mut self, child: &'static Interface) -> Self {
        self.child = Some(child);
        self
    }

    pub const fn ifaces(mut self, ifaces: &'static [&'static Interface]) -> Self {
        self.arg_interfaces = ifaces;
        self
    }

    pub const fn destructor(mut self, v: bool) -> Self {
        self.is_destructor = v;
        self
    }
}

#[macro_export]
macro_rules! dispatch {
    ($kind:ident: $fn_name:ident($($arg:expr),* $(,)*) -> *mut $ty:ty) => {{
        let res = $crate::dispatch!(@@@call $fn_name($($arg,)*));
        if res.is_null() {
            Err($crate::Error::new($crate::ErrorKind::$kind))
        } else {
            Ok(res)
        }
    }};
    ($kind:ident: $fn_name:ident($($arg:expr),* $(,)*) -> $ty:ty) => {{
        let res = $crate::dispatch!(@@@call $fn_name($($arg,)*));
        if res == -1 {
            Err($crate::Error::new($crate::ErrorKind::$kind))
        } else {
            Ok(())
        }
    }};
    (@@@call $fn_name:ident($($arg:expr),* $(,)*)) => {{
        unsafe {$crate::wayland::bindings::$fn_name($($arg,)*)}
        }
    };
}

pub struct EventsReadGuard {
    queued: Arc<QueuedWayland>,
    dpy: NonNull<bindings::wl_display>,
    done: bool,
    //_marker: PhantomData<&'a QueuedWayland>,
}

impl EventsReadGuard {
    pub fn conn_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(bindings::wl_display_get_fd(self.dpy.as_ptr())) }
    }

    pub fn read(mut self) -> crate::Result<usize> {
        self.read_non_dispatch()?;
        Arc::clone(&self.queued).dispatch_pending()
    }

    pub fn read_non_dispatch(&mut self) -> crate::Result<()> {
        self.done = true;
        let ret = unsafe { bindings::wl_display_read_events(self.dpy.as_ptr()) };
        if ret < 0 {
            Err(crate::Error::new(crate::ErrorKind::DispatchError))
        } else {
            Ok(())
        }
    }
}

impl Drop for EventsReadGuard {
    fn drop(&mut self) {
        if !self.done {
            unsafe {
                bindings::wl_display_cancel_read(self.dpy.as_ptr());
            }
        }
    }
}

pub struct QueuedWayland {
    inner: Mutex<Inner>,
    queue: Mutex<EventQueueInner>,
}

pub type WeakHandle = Weak<QueuedWayland>;
pub type StrongHandle = Arc<QueuedWayland>;

impl QueuedWayland {
    pub(crate) fn connect(stream: UnixStream) -> crate::Result<Arc<Self>> {
        let mut inner = Inner::connect(stream)?;
        let eq = inner.inner_queue()?;
        let inner = Mutex::new(inner);
        Ok(Arc::new(Self {
            inner,
            queue: Mutex::new(eq),
        }))
    }

    pub(crate) fn dispatch_pending(self: Arc<Self>) -> crate::Result<usize> {
        let (display, evq) = {
            let guard = self.inner.lock().unwrap();
            (guard.display, self.queue.lock().unwrap().eq)
        };

        let ret = WAYLAND.set(&self, || unsafe {
            bindings::wl_display_dispatch_queue_pending(display.as_ptr(), evq.as_ptr())
        });

        if ret < 0 {
            return Err(crate::Error::new(crate::ErrorKind::DispatchError));
        } else {
            Ok(ret as usize)
        }
    }

    pub(crate) fn lock_read(self: Arc<Self>) -> crate::Result<EventsReadGuard> {
        let (dpy, evq) = {
            let guard = self.inner.lock().unwrap();
            (guard.display, self.queue.lock().unwrap().eq)
        };

        let ret = unsafe { bindings::wl_display_prepare_read_queue(dpy.as_ptr(), evq.as_ptr()) };
        if ret < 0 {
            return Err(crate::Error::new(crate::ErrorKind::DispatchError));
        } else {
            Ok(EventsReadGuard {
                dpy,
                done: false,
                queued: self,
            })
        }
    }

    pub(crate) fn send_message(
        self: Arc<Self>,
        message: Event<ObjectId, RawFd>,
        data: Option<Arc<dyn ObjectData>>,
        child_spec: Option<(&'static Interface, u32)>,
    ) -> crate::Result<ObjectId> {
        let mut guard = self.inner.lock().unwrap();
        let eq_guard = self.queue.lock().unwrap();
        guard.send_message_inner(message, data, child_spec, eq_guard)
    }

    pub(crate) fn object_info(&self, id: ObjectId) -> crate::Result<ObjectMetadata> {
        if !id
            .alive
            .as_ref()
            .map(|a| a.load(Ordering::Acquire))
            .unwrap_or(true)
            || id.proxy.is_none()
        {
            return Err(crate::ErrorKind::InvalidId.into_error());
        }

        let version = if id.id == 1 {
            1
        } else {
            unsafe { bindings::wl_proxy_get_version(id.proxy.into_ptr()) }
        };
        Ok(ObjectMetadata {
            iface: id.iface,
            id,
            version,
        })
    }

    pub(crate) fn object_data(&self, id: ObjectId) -> crate::Result<Arc<dyn ObjectData>> {
        if !id
            .alive
            .as_ref()
            .map(|a| a.load(Ordering::Acquire))
            .unwrap_or(false)
        {
            return Err(crate::ErrorKind::InvalidId.into_error());
        }

        if id.id == 1 {
            return Ok(Arc::new(DummyObjectData));
        }

        let udata =
            unsafe { &*(bindings::wl_proxy_get_user_data(id.proxy.into_ptr()) as *mut ProxyData) };
        Ok(udata.data.clone().unwrap())
    }

    pub fn send_request<P>(
        self: Arc<Self>,
        proxy: &P,
        request: P::Req<'_>,
        data: Option<Arc<dyn ObjectData>>,
    ) -> crate::Result<ObjectId>
    where
        P: Protocol,
    {
        let (msg, child_spec) = proxy.write(&self, request)?;
        let msg = msg.map_fd(|fd| fd.as_raw_fd());
        self.send_message(msg, data, child_spec)
    }
}

scoped_tls::scoped_thread_local!(static WAYLAND: Arc<QueuedWayland>);

pub(super) unsafe extern "C" fn __dispatcher(
    _: *const ffi::c_void,
    proxy: *mut ffi::c_void,
    opcode: u32,
    _: *const bindings::wl_message,
    args: *mut bindings::wl_argument,
) -> ffi::c_int {
    let proxy = proxy as *mut bindings::wl_proxy;
    let p_udata = unsafe { bindings::wl_proxy_get_user_data(proxy) as *mut ProxyData };
    let udata = unsafe { &mut *p_udata };

    let interface = udata.interface;
    let message = match interface.events.get(opcode as usize) {
        Some(evt) => evt,
        None => return -1,
    };

    let mut parsed_args: Vec<Arg<ObjectId, OwnedFd>> = Vec::with_capacity(message.sig.len());
    let mut arg_interfaces = message.arg_interfaces.iter().copied();
    let mut created = None;

    for (i, kind) in message.sig.iter().enumerate() {
        use Arg::*;
        use ArgKind as K;

        match *kind {
            K::Uint => parsed_args.push(Uint(unsafe { (*args.add(i)).u })),
            K::Int => parsed_args.push(Int(unsafe { (*args.add(i)).i })),
            K::Fixed => parsed_args.push(Fixed(unsafe { (*args.add(i)).f })),
            K::Fd => {
                parsed_args.push(Fd(unsafe { OwnedFd::from_raw_fd((*args.add(i)).h) }));
            }
            K::Array => {
                let array = unsafe { &*((*args.add(i)).a) };
                let content =
                    unsafe { std::slice::from_raw_parts(array.data as *const u8, array.size) };
                parsed_args.push(Array(Box::from(content)));
            }
            K::Str(_) => {
                let string = unsafe { (*args.add(i)).s };
                if !string.is_null() {
                    let cstr = CStr::from_ptr(string);
                    parsed_args.push(Str(Some(Box::from(cstr))));
                } else {
                    parsed_args.push(Str(None));
                }
            }
            K::Object(_) => {
                let obj = unsafe { (*args.add(i)).o as *mut bindings::wl_proxy };

                match NonNull::new(obj) {
                    Some(ptr) => {
                        let obj_id = unsafe { bindings::wl_proxy_get_id(obj) };
                        let next_interface = arg_interfaces.next().unwrap_or(&Interface::ANON);
                        let listener = unsafe { bindings::wl_proxy_get_listener(obj) };

                        if std::ptr::eq(listener, &raw const RUST_MANAGED as *const _) {
                            let obj_udata = unsafe {
                                &*(bindings::wl_proxy_get_user_data(obj) as *mut ProxyData)
                            };

                            if !same_interface(next_interface, obj_udata.interface) {
                                return -1;
                            }

                            parsed_args.push(Object(ObjectId {
                                alive: Some(Arc::clone(&obj_udata.alive)),
                                proxy: Some(ptr),
                                id: obj_id,
                                iface: obj_udata.interface,
                            }));
                        } else {
                            parsed_args.push(Object(ObjectId {
                                alive: None,
                                id: obj_id,
                                proxy: ptr.into(),
                                iface: next_interface,
                            }));
                        }
                    }
                    None => parsed_args.push(Object(ObjectId::NIL_OBJECT_ID)),
                }
            }
            K::New => {
                let obj = unsafe { (*args.add(i)).o as *mut bindings::wl_proxy };

                match NonNull::new(obj) {
                    Some(proxy) => {
                        let child_iface = message.child.unwrap_or(&Interface::ANON);
                        let child_alive = Arc::new(AtomicBool::new(true));
                        let child_id = ObjectId {
                            alive: Some(Arc::clone(&child_alive)),
                            id: unsafe { bindings::wl_proxy_get_id(obj) },
                            iface: child_iface,
                            proxy: Some(proxy),
                        };

                        let child_udata = Box::into_raw(Box::new(ProxyData {
                            alive: child_alive,
                            data: Some(Arc::new(DummyObjectData)),
                            interface: child_iface,
                        }));

                        created = Some((child_id.clone(), child_udata));

                        unsafe {
                            bindings::wl_proxy_add_dispatcher(
                                obj,
                                Some(__dispatcher),
                                &raw const RUST_MANAGED as *const _,
                                child_udata as *mut _,
                            );
                        }
                        parsed_args.push(New(child_id));
                    }
                    None => parsed_args.push(Object(ObjectId::NIL_OBJECT_ID)),
                }
            }
        }
    }

    let proxy_id = unsafe { bindings::wl_proxy_get_id(proxy) };
    let id = ObjectId {
        alive: Some(Arc::clone(&udata.alive)),
        proxy: NonNull::new(proxy),
        id: proxy_id,
        iface: udata.interface,
    };

    let ret = WAYLAND.with(|wl| {
        let mut guard = wl.queue.lock().unwrap();

        if let Some((ref new_id, _)) = created {
            guard.known_proxies.insert(new_id.proxy.into_ptr());
        }

        if message.is_destructor {
            guard.known_proxies.remove(&proxy);
        }

        std::mem::drop(guard);
        if let Some(data) = udata.data.clone() {
            data.on_event(
                wl,
                event::Event {
                    sender: id.clone(),
                    opcode: opcode as u16,
                    args: parsed_args,
                },
            )
        } else {
            None
        }
    });

    match (created, ret) {
        (Some((_, child_udata_ptr)), Some(child_udata)) => unsafe {
            (*child_udata_ptr).data = Some(child_udata);
        },
        _ => {
            return -1;
        }
    }
    0
}

unsafe fn free_arrays(sig: &[ArgKind], args: &mut [wl_argument]) {
    for (kind, arg) in sig.iter().zip(args.iter_mut()) {
        if let ArgKind::Array = kind {
            let _ = Box::from_raw(arg.a);
        }
    }
}
