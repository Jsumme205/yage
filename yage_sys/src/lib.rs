mod egl;

pub mod wayland;

pub(crate) mod macros {

    macro_rules! impl_protocol_struct {
        ($vis:vis $Name:ident {
            iface: $INTERFACE:ident,
            request: $Request:ty,
            event: $Event:ty
        }
        ) => {
            $vis struct $Name {
                id: $crate::wl::ObjectId,
                version: u32,
                data: Option<::std::sync::Arc<dyn $crate::wl::ObjectData>>,
                handle: $crate::wl::WeakHandle,
            }


            impl $crate::wl::Protocol for $Name {
                type Req<'request> = $Request;
                type Event = $Event;


                fn iface() -> &'static $crate::wl::Interface {
                    &$INTERFACE
                }

                fn id(&self) -> $crate::wl::ObjectId {
                    self.id.clone()
                }

                fn version(&self) -> u32 {
                    self.version
                }

                fn data<U: Send + Sync + 'static>(&self) -> Option<&U> {
                    self.data
                        .as_ref()
                        .map(|data| data.as_any().downcast_ref())
                        .flatten()
                }

                fn object_data(&self) -> Option<std::sync::Arc<dyn crate::wl::ObjectData>> {
                    self.data.clone()
                }

                fn backend(&self) -> std::sync::Weak<super::QueuedWayland> {
                    self.handle.clone()
                }

                fn from_id(
                    conn: std::sync::Arc<super::QueuedWayland>,
                    id: $crate::wl::ObjectId,
                ) -> crate::Result<Self> {
                    if !$crate::wl::same_interface(Self::iface(), id.iface) && !id.is_null() {
                        return Err(crate::ErrorKind::InvalidId.into_error());
                    }
                    let version = conn
                        .object_info(id.clone())
                        .map(|info| info.version)
                        .unwrap_or(0);
                    let data = conn.object_data(id.clone()).ok();
                    Ok(Self {
                        id,
                        data,
                        version,
                        handle: Arc::downgrade(&conn),
                    })
                }

                fn inert(conn: std::sync::Weak<super::QueuedWayland>) -> Self {
                    Self {
                        id: $crate::wl::ObjectId::NIL_OBJECT_ID,
                        version: 0,
                        data: None,
                        handle: conn,
                    }
                }


                fn cons<P>(&self, req: Self::Req<'_>, data: Arc<dyn crate::wl::ObjectData>) -> crate::Result<P>
                where
                    P: Protocol,
                {
                    let conn = self
                        .handle
                        .upgrade()
                        .ok_or(crate::ErrorKind::InvalidId.into_error())?;
                    let id = std::sync::Arc::clone(&conn).send_request(self, req, Some(data))?;
                    P::from_id(conn, id)
                }

                fn send_request(&self, req: Self::Req<'_>) -> crate::Result<()> {
                    let conn = self
                        .handle
                        .upgrade()
                        .ok_or(crate::ErrorKind::InvalidId.into_error())?;
                    let id = conn.send_request(self, req, None)?;
                    if !id.is_null() {
                        panic!("assertion failed id.is_null()")
                    }
                    Ok(())
                }

            }

            impl ::core::fmt::Debug for $Name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
                {
                    f.debug_struct(stringify!($Name)).finish_non_exhaustive()
                }
            }

        };

    }

    pub(crate) use impl_protocol_struct as protocol_struct;
}

pub mod wl {

    pub use super::wayland::{
        data::{DummyObjectData, ObjectData},
        event::{Arg, Event},
        event_loop::{EventQueue, WeakQueue},
        interfaces,
        protocol::Protocol,
        same_interface, EventsReadGuard, QueuedWayland, WeakHandle,
    };
    pub use super::wayland::{ArgKind, Interface, Message, ObjectId, Proxy, ProxyData};
}

mod xdg;

pub mod errors;

pub use errors::{Error, ErrorKind, Result};
