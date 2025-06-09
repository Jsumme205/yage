use std::sync::Arc;

use super::interfaces::*;
use super::protocol::Protocol;
use crate::macros::protocol_struct;

#[allow(unused)]
pub(super) mod __prelude {
    pub use crate::{
        wayland::protocol::{ProtocolEvent, ProtocolRequest},
        wl::{interfaces::*, Arg, Event as Evt, ObjectId, Protocol},
    };
    pub use std::{convert::Infallible, marker::PhantomData};
    pub use yage_util::IterExt;
}

pub(crate) mod helpers {
    use crate::{
        wayland::StrongHandle,
        wl::{Arg, Interface, ObjectId},
    };

    pub(crate) fn make<const N: usize, Fd>(
        conn: &StrongHandle,
        iface: &'static Interface,
        id: ObjectId,
        args: [Arg<ObjectId, Fd>; N],
    ) -> crate::Result<(Option<(&'static Interface, u32)>, Vec<Arg<ObjectId, Fd>>)> {
        let child_spec = {
            let my_info = conn.object_info(id.clone())?;
            Some((iface, my_info.version))
        };
        let args = {
            let vec = Vec::from(args);
            vec
        };
        Ok((child_spec, args))
    }
}

protocol_struct! {
    pub WlDisplay {
        iface: WL_DISPLAY_INTERFACE,
        request: display::Request<'request>,
        event: display::Event
    }
}

pub mod display {
    use super::__prelude::*;

    use super::helpers;

    pub enum Request<'req> {
        Sync,
        GetRegistry,
        #[doc(hidden)]
        __Phantom {
            _lt_capture: PhantomData<&'req ()>,
            _never_construct: Infallible,
        },
    }

    pub enum Event {
        Error {
            id: ObjectId,
            code: u32,
            message: String,
        },
        Delete {
            id: u32,
        },
    }

    impl ProtocolEvent for Event {
        type Protocol = super::WlDisplay;

        fn parse<I>(_protocol: &Self::Protocol, opcode: u16, iter: I) -> crate::Result<Self>
        where
            I: IntoIterator<Item = crate::wl::Arg<ObjectId, std::os::unix::prelude::OwnedFd>>,
        {
            let mut arg_iter = iter.into_iter();

            match opcode {
                0u16 => {
                    if let (Some(Arg::Object(id)), Some(Arg::Uint(code)), Some(Arg::Str(message))) =
                        (arg_iter.next(), arg_iter.next(), arg_iter.next())
                    {
                        let event = Event::Error {
                            id,
                            code,
                            message: String::from_utf8_lossy(message.as_ref().unwrap().to_bytes())
                                .into_owned(),
                        };
                        return Ok(event);
                    } else {
                        return Err(crate::ErrorKind::SubmitError.into_error());
                    }
                }
                1u16 => {
                    if let Some(Arg::Uint(id)) = arg_iter.next() {
                        return Ok(Event::Delete { id });
                    } else {
                        return Err(crate::ErrorKind::SubmitError.into_error());
                    }
                }
                _ => return Err(crate::ErrorKind::SubmitError.into_error()),
            }
        }
    }

    impl<'req> ProtocolRequest<'req> for Request<'req> {
        type Protocol = super::WlDisplay;

        fn write(
            self,
            protocol: &Self::Protocol,
            conn: &crate::wayland::StrongHandle,
        ) -> crate::Result<(
            crate::wl::Event<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>,
            Option<(&'static crate::wl::Interface, u32)>,
        )> {
            match self {
                Request::Sync => {
                    let (child_spec, args) = helpers::make(
                        conn,
                        &WL_REGISTRY_INTERFACE,
                        protocol.id(),
                        [Arg::New(ObjectId::NIL_OBJECT_ID)],
                    )?;
                    Ok((
                        Evt {
                            sender: protocol.id.clone(),
                            opcode: 0u16,
                            args,
                        },
                        child_spec,
                    ))
                }
                Request::GetRegistry => {
                    let (child_spec, args) = helpers::make(
                        conn,
                        &WL_REGISTRY_INTERFACE,
                        protocol.id(),
                        [Arg::New(ObjectId::NIL_OBJECT_ID)],
                    )?;
                    Ok((
                        Evt {
                            sender: protocol.id.clone(),
                            opcode: 1,
                            args,
                        },
                        child_spec,
                    ))
                }
                Request::__Phantom {
                    _never_construct: never,
                    ..
                } => match never {},
            }
        }
    }
}

protocol_struct! {
    pub WlRegistry {
        iface: WL_REGISTRY_INTERFACE,
        request: registry::Request<'request>,
        event: registry::Event
    }
}

pub mod registry {
    use super::__prelude::*;

    use crate::wl::Interface;

    pub enum Request<'request> {
        Bind {
            name: u32,
            iface: &'static Interface,
            version: u32,
        },
        #[doc(hidden)]
        __Phantom {
            _marker: PhantomData<&'request ()>,
            _never_made: Infallible,
        },
    }

    pub enum Event {
        Global {
            name: u32,
            iface: String,
            version: u32,
        },
        Remove {
            name: u32,
        },
    }

    impl ProtocolEvent for Event {
        type Protocol = super::WlRegistry;

        fn parse<I>(protocol: &Self::Protocol, opcode: u16, iter: I) -> crate::Result<Self>
        where
            I: IntoIterator<Item = Arg<ObjectId, std::os::unix::prelude::OwnedFd>>,
        {
            let mut arg_iter = iter.into_iter();

            match opcode {
                0u16 => {
                    if let Some([Arg::Uint(u), Arg::Str(s), Arg::Uint(v)]) =
                        IterExt::next_chunk(&mut arg_iter)
                    {
                        let event = Event::Global {
                            name: u,
                            iface: String::from_utf8_lossy(s.as_ref().unwrap().to_bytes())
                                .into_owned(),
                            version: v,
                        };
                        return Ok(event);
                    } else {
                        return Err(crate::ErrorKind::SubmitError.into_error());
                    }
                }
                1u16 => {}
            }
        }
    }
}
