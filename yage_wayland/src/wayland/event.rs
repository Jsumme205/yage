use std::ffi::CStr;
//use yage_util::inline::Ssv;

/// TODO: change `args` to `Ssv` when completed
pub struct Event<Id, Fd> {
    pub sender: Id,
    pub opcode: u16,
    pub args: Vec<Arg<Id, Fd>>,
}

impl<Id, Fd> Event<Id, Fd> {
    pub fn map_fd<T>(self, mut f: impl FnMut(Fd) -> T) -> Event<Id, T> {
        Event {
            sender: self.sender,
            opcode: self.opcode,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.__map_fd_inner(&mut f))
                .collect(),
        }
    }
}

pub enum Arg<Id, Fd> {
    Int(i32),
    Uint(u32),
    Fixed(i32),
    Str(Option<Box<CStr>>),
    Object(Id),
    New(Id),
    Array(Box<[u8]>),
    Fd(Fd),
}

impl<Id, Fd> Arg<Id, Fd> {
    fn __map_fd_inner<F, T>(self, f: &mut F) -> Arg<Id, T>
    where
        F: FnMut(Fd) -> T,
    {
        use Arg::*;
        match self {
            Int(i) => Int(i),
            Uint(u) => Uint(u),
            Fixed(f) => Fixed(f),
            Str(s) => Str(s),
            Object(i) => Object(i),
            New(i) => New(i),
            Array(a) => Array(a),
            Fd(fd) => Fd((f)(fd)),
        }
    }
}
