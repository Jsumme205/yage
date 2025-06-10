use super::init::{Init, InitContext};
use super::new::SearchPaths;
use super::TrackedPlugin;
use crate::machine_cog::{Cog, MachineInput, TupleHelper};
use crate::plugin::Plugin;
use crate::states::BuildConfigs;
use core::net::SocketAddr;
use core::pin::Pin;
// TODO: this will break if `alloc` is turned off
use crate::asset::{Cache, Loader};
use core::num::NonZero;
use core::task::Poll;

pub struct LoaderContext<Fs, Net, C> {
    pub(super) filesystem: Fs,
    pub(super) networking: Net,
    #[cfg(feature = "alloc")]
    pub(super) asset_cache: alloc::sync::Arc<C>,
}

pub struct Loading<L, Fs, Net, C> {
    pub(super) loader_plugin: TrackedPlugin<L, LoaderContext<Fs, Net, C>>,
    pub(super) loader_context: LoaderContext<Fs, Net, C>,
    pub(crate) cfgs: BuildConfigs,
}

crate::seal!(Loading<L, Fs, Net, C>);

impl<L, Fs, Net, I, E, C> Cog<(I,)> for Loading<L, Fs, Net, C>
where
    Fs: for<'a> Loader<SearchPaths, &'a str, C, Error = E>,
    Net: Loader<SocketAddr, SocketAddr, C, Error = E>,
    L: for<'a> Plugin<&'a mut LoaderContext<Fs, Net, C>, Output = I, Error = E>,
    C: Cache<NonZero<usize>>,
{
    type Input = super::BuildConfigs;
    type Output<N: TupleHelper> = Init<N::E1>;
    type Error = E;

    fn poll_transform(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        _: &mut MachineInput<'_, Self::Input>,
        _: crate::machine_cog::OnlyCalledByThisCrate,
    ) -> core::task::Poll<Result<Self::Output<(I,)>, Self::Error>> {
        // SAFETY: we're going to be very careful with this
        let Self {
            loader_plugin,
            loader_context,
            ..
        } = unsafe { self.get_unchecked_mut() };
        let loader_plugin = unsafe { Pin::new_unchecked(loader_plugin) };
        match loader_plugin.poll_plugin(cx, loader_context) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(out)) => Poll::Ready(Ok(Init {
                init_plugin: TrackedPlugin::new(out),
                init_context: InitContext {
                    window_width: 0,
                    window_height: 0,
                    window_name: "".into(),
                    _priv: (),
                },
                _marker: core::marker::PhantomData,
            })),
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        }
    }
}
