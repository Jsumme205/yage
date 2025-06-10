use super::main_loop::MainLoop;
use super::TrackedPlugin;
use crate::machine_cog::{Cog, MachineInput, TupleHelper};
use crate::plugin::Plugin;
use core::marker::PhantomData;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct InitContext {
    pub window_width: u32,
    pub window_height: u32,
    #[cfg(feature = "alloc")]
    pub window_name: alloc::string::String,
    pub(super) _priv: (),
}

pub struct Init<I> {
    pub(super) init_plugin: TrackedPlugin<I, InitContext>,
    pub(super) init_context: InitContext,
    pub(super) _marker: PhantomData<fn(InitContext)>,
}

crate::seal!(Init<N>);

impl<I, S, M, E, Eq, R> Cog<(M, S, Eq, R)> for Init<I>
where
    I: for<'a> Plugin<&'a mut InitContext, Output = (M, S, Eq), Error = E>,
{
    type Input = ();
    type Error = E;
    type Output<N: TupleHelper> = MainLoop<N::E1, N::E2, N::E3, N::E4>;

    fn poll_transform(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        _: &mut MachineInput<'_, Self::Input>,
        _: crate::machine_cog::OnlyCalledByThisCrate,
    ) -> Poll<Result<Self::Output<(M, S, Eq, R)>, Self::Error>> {
        let Self {
            init_plugin,
            init_context,
            ..
        } = unsafe { self.get_unchecked_mut() };
        let init_plugin = unsafe { Pin::new_unchecked(init_plugin) };
        let (main_loop, state, event_queue) =
            core::task::ready!(init_plugin.poll_plugin(cx, init_context))?;
        Ok(MainLoop::new(main_loop, state, event_queue))
        .into()
    }
}
