use super::{bindings, data::ObjectData, Interface, ObjectId, Proxy, ProxyData};
use std::{
    collections::HashSet,
    ffi::c_void,
    ptr::NonNull,
    sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard, Weak},
};

pub(super) static RUST_MANAGED: u32 = 0xdeadbeef;

pub struct EventQueue {
    inner: Arc<Mutex<EventQueueInner>>,
}

impl EventQueue {
    pub(crate) unsafe fn new(queue: *mut bindings::wl_event_queue) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EventQueueInner {
                eq: NonNull::new_unchecked(queue),
                known_proxies: HashSet::new(),
            })),
        }
    }

    pub(crate) fn manage(&self, proxy: Proxy) -> ObjectId {
        let Proxy { id, data } = proxy;
        unsafe {
            let mut guard = self.inner.lock().unwrap();
            bindings::wl_proxy_set_queue(id.proxy.unwrap().as_ptr(), guard.eq.as_ptr());
            guard.__manage_object(id.iface, id.proxy.unwrap().as_ptr(), data.data.unwrap())
        }
    }
}

impl EventQueue {
    pub(super) fn lock(&self) -> MutexGuard<'_, EventQueueInner> {
        self.inner.lock().unwrap()
    }
}

pub struct WeakQueue {
    inner: Weak<Mutex<EventQueueInner>>,
}

impl WeakQueue {
    pub fn upgrade(&self) -> Option<EventQueue> {
        Weak::upgrade(&self.inner).map(|inner| EventQueue { inner })
    }
}

pub(super) struct EventQueueInner {
    pub(super) eq: NonNull<bindings::wl_event_queue>,
    pub(super) known_proxies: HashSet<*mut bindings::wl_proxy>,
}

impl EventQueueInner {
    pub(super) unsafe fn __manage_object(
        &mut self,
        iface: &'static Interface,
        proxy: *mut bindings::wl_proxy,
        data: Arc<dyn ObjectData>,
    ) -> ObjectId {
        let alive = Arc::new(AtomicBool::new(true));
        let id = ObjectId {
            proxy: NonNull::new(proxy),
            alive: Some(alive.clone()),
            id: unsafe { bindings::wl_proxy_get_id(proxy) },
            iface,
        };

        self.known_proxies.insert(proxy);
        let udata = Box::into_raw(Box::new(ProxyData {
            alive,
            data: Some(data),
            interface: iface,
        })) as *mut c_void;

        unsafe {
            bindings::wl_proxy_add_dispatcher(
                proxy,
                Some(super::__dispatcher),
                (&raw const RUST_MANAGED) as *const _,
                udata,
            );
        }

        id
    }
}
