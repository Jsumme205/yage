pub mod channel;

use yage_util::atomic::Atomic;

use std::{
    future::Future,
    sync::{atomic::AtomicBool, Arc},
    task::{Wake, Waker},
};

pub use channel::{channel, Reciever, Sender};

pub trait DerivedFromEvent<T> {
    fn from_event(evt: T) -> Self;
}

pub trait ListenerCallback {
    type Event;

    fn on_event_recieved(&mut self, event: Self::Event);

    fn attach<L>(&self, listener: L) -> Arc<Manager<Self::Event, L>>
    where
        L: ListenerCallback<Event: DerivedFromEvent<Self::Event>>;
}

pub struct Manager<E, L>
where
    L: ListenerCallback<Event: DerivedFromEvent<E>>,
{
    listener: Atomic<L>,
    recv: Reciever<E>,
}

impl<E, L> Wake for Manager<E, L>
where
    L: ListenerCallback<Event: DerivedFromEvent<E>>,
{
    fn wake(self: Arc<Self>) {
        let mut listener = self.listener.borrow_mut();
        let event: L::Event = <L::Event as DerivedFromEvent<E>>::from_event(self.recv.recv());
        listener.on_event_recieved(event);
    }
}
