use super::loading::{LoaderContext, Loading};
use super::TrackedPlugin;
use crate::asset::Cache;
use crate::asset::Loader;
use crate::machine_cog::{Cog, TupleHelper};
use core::future::Future;
use core::net::SocketAddr;
use core::num::NonZero;
use core::pin::Pin;
use core::task::Poll;

pub struct New<F, N, E, C, FFut, NFut> {
    state: State<F, N, E, C, FFut, NFut>,
}

enum State<F, N, E, C, FFut, NFut> {
    None,
    Polling {
        file_fut: Option<FFut>,
        network_fut: Option<NFut>,
        file: Option<F>,
        net: Option<N>,
    },
    Done {
        loader: Result<(F, N, C), E>,
    },
    Panic,
}

crate::seal!(New<F, N, E, FFut, NFut, C>);

pub struct SearchPaths {
    #[cfg(feature = "alloc")]
    pub paths: alloc::vec::Vec<&'static str>,
    #[cfg(not(feature = "alloc"))]
    pub paths: (),
}

impl Clone for SearchPaths {
    fn clone(&self) -> Self {
        SearchPaths {
            paths: self.paths.clone(),
        }
    }
}

impl<F, N, E, C, Fut, Net> New<F, N, E, C, Fut, Net> {
    pub(crate) const fn new() -> Self {
        Self { state: State::None }
    }
}

impl<E, L, F, N, Fut, C> Cog<(L,)> for New<F, N, E, C, Fut, N::InitFuture>
where
    F: for<'a> Loader<SearchPaths, &'a str, C, Error = E, InitFuture = Fut>,
    N: Loader<SocketAddr, SocketAddr, C, Error = E>,
    Fut: Future<Output = Result<F, E>>,
    C: Cache<NonZero<usize>> + From<SearchPaths>,
{
    type Input = (super::BuildConfigs, Option<L>);
    type Output<O: TupleHelper> = Loading<<O as TupleHelper>::E1, F, N, C>;
    type Error = E;

    fn poll_transform(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        input: &mut crate::machine_cog::MachineInput<'_, Self::Input>,
        _: crate::machine_cog::OnlyCalledByThisCrate,
    ) -> core::task::Poll<Result<Self::Output<(L,)>, Self::Error>> {
        let Self { state } = unsafe { self.get_unchecked_mut() };
        #[cfg(feature = "alloc")]
        let paths = core::mem::take(&mut input.input.0.search_paths);
        #[cfg(not(feature = "alloc"))]
        let paths = ();

        let search_paths = SearchPaths { paths };
        match state {
            State::None => {
                let file_fut = F::init(search_paths.clone());
                let net_fut = N::init(input.input.0.addr);
                *state = State::Polling {
                    file_fut: Some(file_fut),
                    network_fut: Some(net_fut),
                    file: None,
                    net: None,
                };
            }
            State::Polling {
                file_fut,
                network_fut,
                file,
                net,
            } => {
                match (file.is_some(), net.is_some()) {
                    (true, true) => {
                        let file = file.take().unwrap();
                        let net = net.take().unwrap();
                        *state = State::Done {
                            loader: Ok((file, net, C::from(search_paths))),
                        };
                        // we have to wake right here and return pending to satisfy borrowcheck
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    _ => {}
                }

                let (file_fut, network_fut) = unsafe {
                    let (t1, t2) = (
                        Pin::new_unchecked(file_fut),
                        Pin::new_unchecked(network_fut),
                    );
                    (t1.as_pin_mut(), t2.as_pin_mut())
                };
                if let Some(file_fut) = file_fut {
                    match file_fut.poll(cx) {
                        core::task::Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(f)) => {
                            *file = Some(f);
                        }
                        Poll::Ready(Err(e)) => {
                            *state = State::Done { loader: Err(e) };
                            // we have to wake right here and return pending to satisfy borrowcheck
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                    }
                }
                if let Some(net_fut) = network_fut {
                    match net_fut.poll(cx) {
                        core::task::Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(f)) => {
                            *net = Some(f);
                        }
                        Poll::Ready(Err(e)) => *state = State::Done { loader: Err(e) },
                    }
                }
            }
            State::Done { .. } => {
                let State::Done { loader } = core::mem::replace(state, State::Panic) else {
                    unreachable!()
                };
                let (filesystem, networking, cache) = loader?;
                return Poll::Ready(Ok(Loading {
                    loader_context: LoaderContext {
                        filesystem,
                        networking,
                        asset_cache: alloc::sync::Arc::new(cache),
                    },
                    loader_plugin: TrackedPlugin::new(input.input.1.take().expect("lmao")),
                    cfgs: core::mem::take(&mut input.input.0),
                }));
            }
            State::Panic => panic!("how did we get here lmao"),
        }
        Poll::Pending
    }
}
