use std::sync::Arc;

use super::interfaces::*;
use super::protocol::Protocol;
use crate::macros::protocol_struct;

macro_rules! submit_err {
    () => {
        Err($crate::errors::ErrorKind::SubmitError.into_error())
    };
}

macro_rules! expect {
    ($variant:ty = $iter:ident => $do:block) => {
        if let Some($variant) = ($iter).next() {
            $do
        } else {
            return submit_err!();
        }
    };
}

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

    impl<'req> ProtocolRequest<'req> for Request<'req> {
        type Protocol = super::WlRegistry;

        fn write(
            self,
            protocol: &Self::Protocol,
            conn: &crate::wayland::StrongHandle,
        ) -> crate::Result<(
            Evt<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>,
            Option<(&'static Interface, u32)>,
        )> {
            use super::helpers;
            match self {
                Self::Bind {
                    name,
                    iface,
                    version,
                } => {
                    let (child_spec, args) = helpers::make(
                        conn,
                        iface,
                        protocol.id(),
                        [
                            Arg::Uint(name),
                            Arg::Str(Some(std::ffi::CString::new(iface.name).into())),
                            Arg::Uint(version),
                        ],
                    )?;
                    Ok((
                        Evt {
                            sender: protocol.id.clone(),
                            opcode: 0,
                            args,
                        },
                        child_spec,
                    ))
                }
                Self::__Phantom {
                    _never_made: never, ..
                } => match never {},
            }
        }
    }

    #[derive(Debug, Clone)]
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

    impl Event {
        pub fn opcode(&self) -> u16 {
            match self {
                Self::Global { .. } => 0u16,
                Self::Remove { .. } => 1u16,
            }
        }
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
                1u16 => {
                    if let Some(Arg::Uint(u)) = arg_iter.next() {
                        let event = Event::Remove { name: u };
                        return Ok(event);
                    } else {
                        return Err(crate::ErrorKind::SubmitError.into_error());
                    }
                }
                _ => Err(crate::errors::ErrorKind::SubmitError.into_error()),
            }
        }
    }
}

protocol_struct! {
    pub WlCallback {
        iface: WL_CALLBACK_IFACE,
        request: callback::Request<'request>,
        event: callback::Event
    }
}

pub mod callback {
    use super::__prelude::*;

    pub enum Request<'req> {
        __Phantom {
            _capture: PhantomData<&'req ()>,
            never: Infallible,
        },
    }

    impl<'req> ProtocolRequest<'req> for Request<'req> {
        type Protocol = super::WlCallback;

        fn write(
            self,
            protocol: &Self::Protocol,
            conn: &crate::wayland::StrongHandle,
        ) -> crate::Result<(
            Evt<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>,
            Option<(&'static crate::wl::Interface, u32)>,
        )> {
            match self {
                Self::__Phantom { never, .. } => match never {},
            }
        }
    }

    pub enum Event {
        Done { callback_data: u32 },
    }

    impl ProtocolEvent for Event {
        type Protocol = super::WlCallback;

        fn parse<I>(protocol: &Self::Protocol, opcode: u16, iter: I) -> crate::Result<Self>
        where
            I: IntoIterator<Item = Arg<ObjectId, std::os::unix::prelude::OwnedFd>>,
        {
            let mut iter = iter.into_iter();
            match opcode {
                0u16 => {
                    if let Some(Arg::Uint(callback_data)) = iter.next() {
                        return Ok(Event::Done { callback_data });
                    }
                    return submit_err!();
                }
                _ => submit_err!(),
            }
        }
    }
}

protocol_struct! {
    pub WlSurface {
        iface: WL_SURFACE_IFACE,
        request: surface::Request<'request>,
        event: surface::Event
    }
}

pub mod surface {
    use super::__prelude::*;

    pub const SURFACE_DESTROY_OPCODE: u16 = 0;
    pub const SURFACE_ATTACH_OPCODE: u16 = 1;
    pub const SURFACE_DAMAGE_OPCODE: u16 = 2;
    pub const SURFACE_FRAME_OPCODE: u16 = 3;
    pub const SURFACE_SET_OPAQUE_REGION_OPCODE: u16 = 4;
    pub const SURFACE_SET_INPUT_REGION_OPCODE: u16 = 5;
    pub const SURFACE_COMMIT_OPCODE: u16 = 6;
    pub const SURFACE_SET_BUFFER_TRANSFORM_OPCODE: u16 = 7;

    pub enum Request<'req> {
        Destroy,
        Attach {
            buffer: ObjectId,
            x_pos: i32,
            y_pos: i32,
        },
        Damage {
            x_pos: i32,
            y_pos: i32,
            width: i32,
            height: i32,
        },
        Frame,
        SetOpaqueRegion {
            region: ObjectId,
        },
        SetInputRegion {
            region: ObjectId,
        },
        Commit,
        SetBufferTransform {
            transform: i32,
        },
        SetBufferScale {
            scale: i32,
        },
        DamageBuffer {
            x_pos: i32,
            y_pos: i32,
        },
        Offset {
            off_x: i32,
            off_y: i32,
        },
        __Phantom {
            never: Infallible,
            _phantom: PhantomData<&'req ()>,
        },
    }

    pub enum Event {
        Enter { output: ObjectId },
        Leave { out: ObjectId },
        PrefferedBufferScale { scale: i32 },
        PrefferedBufferTransform { transform: u32 },
    }

    impl ProtocolEvent for Event {
        type Protocol = super::WlSurface;

        fn parse<I>(protocol: &Self::Protocol, opcode: u16, iter: I) -> crate::Result<Self>
        where
            I: IntoIterator<Item = Arg<ObjectId, std::os::unix::prelude::OwnedFd>>,
        {
            let mut iter = iter.into_iter();
            match opcode {
                0u16 => {
                    expect! {
                        Arg::Object(output) = iter => {
                            return Ok(Event::Enter { output });
                        }
                    };
                }
                1u16 => {
                    expect! {
                        Arg::Object(output) = iter => {
                            return Ok(Event::Leave { out: output });
                        }
                    }
                }
                2u16 => {
                    expect! {
                        Arg::Int(scale) = iter => {
                            return Ok(Event::PrefferedBufferScale { scale });
                        }
                    }
                }
                3u16 => {
                    expect! {
                        Arg::Uint(transform) = iter =>  {
                            return Ok(Event::PrefferedBufferTransform { transform });
                        }
                    }
                }
                _ => submit_err!(),
            }
        }
    }

    impl<'req> ProtocolRequest<'req> for Request<'req> {
        type Protocol = super::WlSurface;

        fn write(
            self,
            protocol: &Self::Protocol,
            conn: &crate::wayland::StrongHandle,
        ) -> crate::Result<(
            Evt<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>,
            Option<(&'static crate::wl::Interface, u32)>,
        )> {
            use super::helpers;
            match self {
                Self::Destroy => {
                    let (child_spec, args): (
                        Option<_>,
                        Vec<Arg<ObjectId, std::os::unix::prelude::BorrowedFd<'req>>>,
                    ) = helpers::make(conn, &WL_SURFACE_IFACE, protocol.id(), [])?;
                    Ok((
                        Evt {
                            sender: protocol.id(),
                            opcode: 0u16,
                            args,
                        },
                        child_spec,
                    ))
                }
                Self::Attach {
                    buffer,
                    x_pos,
                    y_pos,
                } => {
                    let (child_spec, args) = helpers::make(
                        conn,
                        &WL_BUFFER_IFACE,
                        protocol.id(),
                        [Arg::Object(buffer), Arg::Int(x_pos), Arg::Int(y_pos)],
                    )?;
                    Ok((
                        Evt {
                            sender: protocol.id(),
                            opcode: 1u16,
                            args,
                        },
                        child_spec,
                    ))
                }
                Self::Damage {
                    x_pos,
                    y_pos,
                    width,
                    height,
                } => {
                    let (child_spec, args) = helpers::make(
                        conn,
                        &WL_SURFACE_IFACE,
                        protocol.id(),
                        [
                            Arg::Int(x_pos),
                            Arg::Int(y_pos),
                            Arg::Int(width),
                            Arg::Int(height),
                        ],
                    )?;
                }
                Self::Frame => {}
                Self::SetOpaqueRegion { region } => {}
                Self::SetInputRegion { region } => {}
                Self::Commit => {}
                Self::SetBufferTransform { transform } => {}
                Self::SetBufferScale { scale } => {}
                Self::DamageBuffer { x_pos, y_pos } => {}
                Self::Offset { off_x, off_y } => {}
                Self::__Phantom { never, .. } => match never {},
            }
        }
    }
}
