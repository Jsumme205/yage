use crate::{
    wayland::WeakHandle,
    wl::{ObjectData, ObjectId, Protocol, QueuedWayland},
};

use super::{
    enums::{XdgBaseEvent, XdgBaseRequest},
    ifaces::XDG_WM_IFACE,
};
use std::{
    fmt,
    sync::{Arc, Weak},
};

pub struct XdgWmBase {
    handle: WeakHandle,
    id: ObjectId,
    object_data: Option<Arc<dyn ObjectData>>,
}

impl fmt::Debug for XdgWmBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XdgWmBase").finish_non_exhaustive()
    }
}

impl Protocol for XdgWmBase {
    type Req<'request> = XdgBaseRequest<'request>;
    type Event = XdgBaseEvent;

    fn iface() -> &'static crate::wl::Interface {
        &XDG_WM_IFACE
    }

    fn id(&self) -> crate::wl::ObjectId {
        self.id.clone()
    }

    fn cons<P>(&self, req: Self::Req<'_>, data: Arc<dyn crate::wl::ObjectData>) -> crate::Result<P>
    where
        P: Protocol,
    {
        todo!()
    }

    fn data<U: Send + Sync + 'static>(&self) -> Option<&U> {
        todo!()
    }

    fn inert(conn: Weak<QueuedWayland>) -> Self {
        Self {
            handle: conn,
            id: ObjectId::NIL_OBJECT_ID,
            object_data: None,
        }
    }

    fn parse(
        conn: &Arc<QueuedWayland>,
        evt: crate::wl::Event<ObjectId, std::os::unix::prelude::OwnedFd>,
    ) -> crate::Result<(Self, Self::Event)> {
        todo!()
    }

    fn write<'req>(
        &self,
        conn: &Arc<QueuedWayland>,
        req: Self::Req<'req>,
    ) -> crate::Result<(
        crate::wl::Event<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>,
        Option<(&'static crate::wl::Interface, u32)>,
    )> {
        todo!()
    }

    fn backend(&self) -> Weak<QueuedWayland> {
        self.handle.clone()
    }

    fn from_id(conn: Arc<QueuedWayland>, id: ObjectId) -> crate::Result<Self> {
        Ok(Self {
            handle: Arc::downgrade(&conn),
            id,
            object_data: None,
        })
    }

    fn version(&self) -> u32 {
        7
    }

    fn object_data(&self) -> Option<Arc<dyn crate::wl::ObjectData>> {
        self.object_data.clone()
    }

    fn send_request(&self, req: Self::Req<'_>) -> crate::Result<()> {
        todo!()
    }
}
