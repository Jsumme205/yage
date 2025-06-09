use core::pin::Pin;
use core::task::{Poll, Context};

/// eventually maybe store arbitrary data types in here. stability issue
pub struct MachineInput<'a, I> {
  pub(crate) input: &'a mut I
}


macro_rules! seal_impl {
    ($ty:ident$(<$($gen:tt),*>)?) => {
        impl $(<$($gen),*>)? $crate::machine_cog::sealed::Sealed for $ty $(<$($gen),*>)? {}
    };
}

pub(crate) use seal_impl as seal;


pub(crate) mod sealed {
  pub trait Sealed {}
  impl<T, E> Sealed for (T, E) {}
  impl<T> Sealed for (T,) {}

  #[derive(Clone, Copy)]
  pub struct OnlyCalledByThisCrate {
    _priv: ()
  }

  impl OnlyCalledByThisCrate {
    pub const VAL: Self = Self {
      _priv: ()
    };
  }
}

pub trait TupleHelper: sealed::Sealed {
  type E1;
  type E2;
}


impl<T, E> TupleHelper for (T, E) {
  type E1 = T;
  type E2 = E;
} 

impl<T> TupleHelper for (T,) {
  type E1 = T;
  type E2 = ();
}

pub trait Cog<Next: TupleHelper>: sealed::Sealed {
  type Input;
  type Output<N: TupleHelper>;
  type Error;

  fn poll_transform(
    self: Pin<&mut Self>, 
    cx: &mut Context<'_>, 
    input: &mut MachineInput<'_, Self::Input>,
    token: sealed::OnlyCalledByThisCrate,
  ) -> Poll<Result<Self::Output<Next>, Self::Error>>;
}
