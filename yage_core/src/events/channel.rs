use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU8, Ordering};

use super::DerivedFromEvent;

// this will block if slot is initialized
struct EventInner<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    // 7: 1: full, 0: taken
    // 6: 1: (logical) sender alive, 0: (logical) sender dropped
    // 5: 1: (logical) reciever alive, 0: (logical) reciever dropped
    stamp: AtomicU8,
}

impl<T> EventInner<T> {
    const TAKEN_NONDROP_STATE: u8 = 0b0110_0000;
    const RECV_MASK: u8 = 0b0010_0000;
    const SEND_MASK: u8 = 0b0100_0000;
    const FULL_MASK: u8 = 0b1000_0000;

    const fn uninit() -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            stamp: AtomicU8::new(Self::TAKEN_NONDROP_STATE),
        }
    }

    unsafe fn set_recv_dropped(&self) {
        self.stamp.fetch_and(!Self::RECV_MASK, Ordering::Acquire);
    }

    unsafe fn set_send_dropped(&self) {
        self.stamp.fetch_and(!Self::SEND_MASK, Ordering::Acquire);
    }

    unsafe fn take_unchecked(&self) -> T {
        self.stamp.fetch_and(!Self::FULL_MASK, Ordering::SeqCst);
        unsafe {
            let v = core::ptr::replace(self.slot.get(), MaybeUninit::uninit());
            v.assume_init()
        }
    }

    fn try_take(&self) -> Option<T> {
        let stamp = self.stamp.load(Ordering::Acquire);
        if stamp & Self::FULL_MASK == 0 {
            return None;
        }
        Some(unsafe { self.take_unchecked() })
    }

    fn take_blocking(&self) -> T {
        loop {
            let stamp = self.stamp.load(Ordering::Acquire);
            if stamp & Self::FULL_MASK != 0 {
                return unsafe { self.take_unchecked() };
            }
            std::hint::spin_loop();
        }
    }

    fn write(&self, val: T) {
        let mut state = self.stamp.load(Ordering::Acquire);

        loop {
            match self.stamp.compare_exchange_weak(
                state,
                state | Self::FULL_MASK,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => unsafe {
                    let p = &mut (*self.slot.get());
                    p.write(val);
                    return;
                },
                Err(s) => state = s,
            }
        }
    }
}

fn allocate_inner<T>() -> NonNull<EventInner<T>> {
    let layout = Layout::new::<EventInner<T>>();
    match NonNull::new(unsafe { std::alloc::alloc(layout) } as *mut _) {
        Some(p) => p,
        None => std::alloc::handle_alloc_error(layout),
    }
}

pub fn channel<T>() -> (Sender<T>, Reciever<T>) {
    let ptr = allocate_inner();
    unsafe {
        ptr.write(EventInner::uninit());
    }
    (
        Sender {
            ptr,
            _marker: PhantomData,
        },
        Reciever {
            ptr,
            _marker: PhantomData,
        },
    )
}

pub struct Sender<T> {
    ptr: NonNull<EventInner<T>>,
    _marker: PhantomData<EventInner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, val: T) {
        unsafe {
            self.ptr.as_ref().write(val);
        }
    }
}

pub struct Reciever<T> {
    ptr: NonNull<EventInner<T>>,
    _marker: PhantomData<EventInner<T>>,
    //_marker2: PhantomData<U>,
}

impl<T> Reciever<T> {
    pub fn recv(&self) -> T {
        unsafe { self.ptr.as_ref().take_blocking() }
    }
}
