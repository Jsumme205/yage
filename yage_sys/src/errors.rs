use core::fmt;
use std::{any::Any, backtrace::Backtrace, error, fmt::Write, sync::Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ErrorKind {
    LoadingFailure,
    NoWayland,
    AllocError(&'static str),
    FlushError,
    DispatchError,
    WouldBlock,
    InvalidId,
    InvalidInterface,
    SubmitError,
    #[default]
    Other,
}

impl ErrorKind {
    pub const fn into_error(self) -> Error {
        Error::new(self)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadingFailure => f.write_str("there was a failure loading `EGL` bindings"),
            Self::NoWayland => f.write_str("the wayland library doesn't seem to be on your system"),
            Self::AllocError(loc) => write!(f, "there was an allocation error at: {}", *loc),
            Self::FlushError => f.write_str("there was an error flushing the display"),
            Self::DispatchError => f.write_str("there was an error while dispatching events"),
            Self::WouldBlock => f.write_str("the operation would block"),
            Self::InvalidId => f.write_str("the id that was given is invalid"),
            Self::InvalidInterface => f.write_str("invalid interface"),
            Self::SubmitError => f.write_str("there was a submission error"),
            Self::Other => f.write_str("there was an unspecified error"),
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    payload: Option<Box<dyn error::Error + Send>>,
    #[cfg(debug_assertions)]
    bt: Option<Backtrace>,
}

impl Error {
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            payload: None,
            #[cfg(debug_assertions)]
            bt: None,
        }
    }

    #[cfg(debug_assertions)]
    pub fn with_capture(kind: ErrorKind) -> Self {
        Self {
            kind,
            payload: None,
            bt: Some(Backtrace::force_capture()),
        }
    }

    pub fn with_payload<T>(self, payload: T) -> Self
    where
        T: Into<Box<dyn error::Error + Send>>,
    {
        Self {
            payload: Some(payload.into()),
            ..self
        }
    }

    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("Error");
        fmt.field("kind", &self.kind);
        if let Some(payload) = self.payload.as_ref() {
            fmt.field("payload", payload);
        }
        #[cfg(debug_assertions)]
        if let Some(backtrace) = self.bt.as_ref() {
            fmt.field("backtrace", backtrace);
        }
        fmt.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
