#![no_std]
#![cfg(feature = "alloc")]
extern crate alloc;

pub mod machine_cog;
pub mod plugin;
pub mod states;

#[cfg(feature = "alloc")]
pub mod asset;

pub mod prelude {
    pub use crate::machine_cog::Cog;
    pub use crate::plugin::{Plugin};
    pub use crate::states::{
        init::{Init, InitContext},
        loading::{LoaderContext, Loading},
        new::New,
        BuildConfigs,
    };

    pub use crate::asset::{Cache, CowHandle, OwnedHandle, BorrowedHandle};
}

macro_rules! token_impl {
    () => {
        $crate::machine_cog::sealed::OnlyCalledByThisCrate::VAL
    };
}

pub(crate) use token_impl as token;

use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) use machine_cog::seal;

use crate::prelude::Cache;

pub struct AppData {
    #[cfg(feature = "alloc")]
    cache: alloc::sync::Arc<dyn Cache<core::num::NonZero<usize>>>
}

pub struct App<S> {
    state: S,
    data: Option<AppData>,
}

struct AppProj<'__pin, S> {
    state: Pin<&'__pin mut S>,
    data: &'__pin mut Option<AppData>,
}

impl<S> App<S> {
    fn project(self: Pin<&mut Self>) -> AppProj<'_, S> {
        unsafe {
            let Self { state, data } = self.get_unchecked_mut();
            AppProj {
                state: Pin::new_unchecked(state),
                data,
            }
        }
    }

    /// NOTE: logically takes ownership of `Self`
    ///
    /// SAFETY:
    /// - must be called in the correct state
    /// - must be called in a function that takes `self` by value
    unsafe fn poll_internal<T: machine_cog::TupleHelper, I, E>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &mut I,
    ) -> Poll<Result<App<S::Output<T>>, E>>
    where
        S: machine_cog::Cog<T, Input = I, Error = E>,
    {
        let AppProj { state, data } = self.project();

        let mut input = machine_cog::MachineInput { input };

        let token = crate::token!();

        let next = core::task::ready!(state.poll_transform(cx, &mut input, token))?;
        
        Poll::Ready(Ok(App {
            state: next,
            data: core::mem::replace(data, None),
        }))
    }
}

use crate::prelude::New;

pub struct Harness<F, A, Cx> {
    fun: F,
    app: A,
    cx: Cx,
}

impl<F, A, Cx> Harness<F, A, Cx> {
    pub fn new(app: A, cx: Cx, fun: F) -> Self {
        Self { fun, app, cx }
    }
}

impl<F, A, Cx, T> core::future::Future for Harness<F, A, Cx>
where
    F: FnMut(&mut Context<'_>, Pin<&mut A>, &mut Cx) -> Poll<T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { fun, app, cx } = unsafe { self.get_unchecked_mut() };
        let app = unsafe { Pin::new_unchecked(app) };
        (fun)(ctx, app, cx)
    }
}

impl<F, N, E, C, FFut> App<New<F, N, E, C, FFut, N::InitFuture>>
where
    F: for<'a> crate::asset::Loader<
        states::new::SearchPaths,
        &'a str,
        C,
        Error = E,
        InitFuture = FFut,
    >,
    N: crate::asset::Loader<core::net::SocketAddr, core::net::SocketAddr, C, Error = E>,
    FFut: core::future::Future<Output = Result<F, E>>,
    C: asset::Cache<core::num::NonZero<usize>> + From<alloc::vec::Vec<&'static str>>,
{
    pub fn new() -> Self {
        Self {
            state: New::new(),
            data: AppData,
        }
    }

    pub async fn load_default<L: Default>(
        self,
        cfg: crate::prelude::BuildConfigs,
    ) -> Result<App<crate::prelude::Loading<L, F, N, C>>, E> {
        Harness::new(
            self,
            (cfg, Some(L::default())),
            |cx: &mut Context<'_>,
                  app: Pin<&mut Self>,
                  cfg: &mut (crate::prelude::BuildConfigs, Option<L>)| unsafe {
                app.poll_internal(cx, cfg)
            },
        )
        .await
    }

    pub async fn load_with<L>(
        self,
        loader: L,
        cfg: crate::prelude::BuildConfigs,
    ) -> Result<App<crate::prelude::Loading<L, F, N, C>>, E> {
        Harness::new(
            self,
            (cfg, Some(loader)),
            |cx: &mut Context<'_>,
                  app: Pin<&mut Self>,
                  cfg: &mut (crate::prelude::BuildConfigs, Option<L>)| unsafe {
                app.poll_internal(cx, cfg)
            },
        )
        .await
    }
}

impl<L, F, N, C> App<crate::prelude::Loading<L, F, N, C>>
{

    pub async fn init<I>(self) -> Result<App<crate::prelude::Init<I>>>
    {
        Harness::new(self, cx, fun)        
    }
    
}
