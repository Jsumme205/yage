use core::convert::Infallible;
use core::pin::Pin;
use core::task::{Context, Poll};

/// eventually maybe store arbitrary data types in here. stability issue
pub struct MachineInput<'a, I> {
    pub(crate) input: &'a mut I,
}

macro_rules! seal_impl {
    ($ty:ident$(<$($gen:tt),*>)?) => {
        impl $(<$($gen),*>)? $crate::machine_cog::sealed::Sealed for $ty $(<$($gen),*>)? {}
    };
}

pub(crate) use seal_impl as seal;

pub(crate) mod sealed {
    pub trait Sealed {}
    impl<T1, T2, T3, T4> Sealed for (T1, T2, T3, T4) {}
    impl<T, E, F> Sealed for (T, E, F) {}
    impl<T, E> Sealed for (T, E) {}
    impl<T> Sealed for (T,) {}
}

#[derive(Clone, Copy)]
pub struct OnlyCalledByThisCrate {
    _priv: (),
}

impl OnlyCalledByThisCrate {
    pub(crate) const VAL: Self = Self { _priv: () };
}

pub trait TupleHelper: sealed::Sealed {
    type E1;
    type E2;
    type E3;
    type E4;
}

impl<T, E, F> TupleHelper for (T, E, F) {
    type E1 = T;
    type E2 = E;
    type E3 = F;
    type E4 = Infallible;
}

impl<T, E> TupleHelper for (T, E) {
    type E1 = T;
    type E2 = E;
    type E3 = Infallible;
    type E4 = Infallible;
}

impl<T> TupleHelper for (T,) {
    type E1 = T;
    type E2 = Infallible;
    type E3 = Infallible;
    type E4 = Infallible;
}

impl<T1, T2, T3, T4> TupleHelper for (T1, T2, T3, T4) {
    type E1 = T1;
    type E2 = T2;
    type E3 = T3;
    type E4 = T4;
}

pub trait Cog<Next: TupleHelper>: sealed::Sealed {
    type Input;
    type Output<N: TupleHelper>;
    type Error;

    fn poll_transform(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &mut MachineInput<'_, Self::Input>,
        token: OnlyCalledByThisCrate,
    ) -> Poll<Result<Self::Output<Next>, Self::Error>>;
}
