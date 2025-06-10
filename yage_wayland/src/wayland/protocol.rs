use core::fmt;
use std::{
    marker::PhantomData,
    os::fd::{BorrowedFd, OwnedFd},
    sync::{Arc, Weak},
};

use crate::wl::{Arg, ObjectData};

use super::{event::Event, Interface, ObjectId, QueuedWayland, StrongHandle};

pub struct WeakProtocolHandle<P> {
    wl: Weak<QueuedWayland>,
    id: ObjectId,
    _marker: PhantomData<P>,
}

impl<P: Protocol> fmt::Debug for WeakProtocolHandle<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(protocol) = self.upgrade() {
            f.debug_struct("Weak").field("protocol", &protocol).finish()
        } else {
            f.debug_struct("Weak").finish_non_exhaustive()
        }
    }
}

impl<P: Protocol> WeakProtocolHandle<P> {
    pub fn upgrade(&self) -> crate::Result<P> {
        let backend = self
            .wl
            .upgrade()
            .ok_or(crate::ErrorKind::InvalidId.into_error())?;
        // backend.info();
        P::from_id(backend, self.id.clone())
    }
}

pub trait Protocol: fmt::Debug + Sized {
    type Event: ProtocolEvent<Protocol = Self>;
    type Req<'request>: ProtocolRequest<'request, Protocol = Self>;

    fn iface() -> &'static Interface;

    fn id(&self) -> ObjectId;
    fn version(&self) -> u32;
    fn data<U: Send + Sync + 'static>(&self) -> Option<&U>;
    fn object_data(&self) -> Option<Arc<dyn ObjectData>>;

    fn backend(&self) -> Weak<QueuedWayland>;
    fn from_id(conn: Arc<QueuedWayland>, id: ObjectId) -> crate::Result<Self>;
    fn inert(conn: Weak<QueuedWayland>) -> Self;

    fn send_request(&self, req: Self::Req<'_>) -> crate::Result<()>;

    fn cons<P>(&self, req: Self::Req<'_>, data: Arc<dyn ObjectData>) -> crate::Result<P>
    where
        P: Protocol;

    fn parse(
        conn: &Arc<QueuedWayland>,
        evt: Event<ObjectId, OwnedFd>,
    ) -> crate::Result<(Self, Self::Event)> {
        let me = Self::from_id(Arc::clone(conn), evt.sender.clone())?;
        let event = <Self::Event as ProtocolEvent>::parse(&me, evt.opcode, evt.args)?;
        Ok((me, event))
    }

    fn write<'req>(
        &self,
        conn: &Arc<QueuedWayland>,
        req: Self::Req<'req>,
    ) -> crate::Result<(
        Event<ObjectId, BorrowedFd<'req>>,
        Option<(&'static Interface, u32)>,
    )> {
        <Self::Req<'req> as ProtocolRequest<'req>>::write(req, self, conn)
    }

    fn is_alive(&self) -> bool {
        true
    }

    fn downgrade(&self) -> WeakProtocolHandle<Self> {
        WeakProtocolHandle {
            wl: self.backend(),
            id: self.id(),
            _marker: PhantomData,
        }
    }
}

pub trait ProtocolEvent: Sized {
    type Protocol: Protocol<Event = Self>;

    fn parse<I>(protocol: &Self::Protocol, opcode: u16, iter: I) -> crate::Result<Self>
    where
        I: IntoIterator<Item = Arg<ObjectId, OwnedFd>>;
}

pub trait ProtocolRequest<'req>: Sized + 'req {
    type Protocol: Protocol<Req<'req> = Self>;

    fn write(
        self,
        protocol: &Self::Protocol,
        conn: &StrongHandle,
    ) -> crate::Result<(
        Event<ObjectId, BorrowedFd<'req>>,
        Option<(&'static Interface, u32)>,
    )>;
}
