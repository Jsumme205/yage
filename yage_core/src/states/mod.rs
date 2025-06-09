
use core::net::SocketAddr;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::plugin::Plugin;

pub mod new;
pub mod loading;
pub mod init;
pub mod main_loop;

#[derive(Debug, Clone, Copy)]
enum State {
  Ready,
  Transform,
  Done
}

pub(crate) struct TrackedPlugin<P, Ctx> {
  state: State,
  plugin: P,
  _ctx: PhantomData<*mut Ctx>,
}

impl<P, Ctx> TrackedPlugin<P, Ctx> {

  pub(crate) fn new(plugin: P) -> Self {
    Self {
      state: State::Ready,
      plugin,
      _ctx: PhantomData
    }
  }
  
}

impl<P, Ctx, E, O> TrackedPlugin<P, Ctx> 
where
  P: for<'a> Plugin<&'a mut Ctx, Output = O, Error = E>
{
  
  pub(crate) fn poll_plugin(self: Pin<&mut Self>, cx: &mut Context<'_>, ctx: &mut Ctx) -> Poll<Result<O, E>> {
    let Self { state, plugin, .. } = unsafe { self.get_unchecked_mut() };
    let mut plugin = unsafe { Pin::new_unchecked(plugin) };
    loop {
      match state {
        State::Ready => {
           match plugin.as_mut().poll_ready(cx, ctx) {
              Poll::Pending => return Poll::Pending,
              Poll::Ready(Err(e)) => {
                *state = State::Done;
                return Poll::Ready(Err(e))
              },
              Poll::Ready(Ok(_)) => {
                *state = State::Transform;
                continue;
              }
           } 
        },
        State::Transform => {
          let out = core::task::ready!(plugin.as_mut().poll_transform(cx))?;
          *state = State::Done;
          return Poll::Ready(Ok(out));
        },
        State::Done => panic!("something went wrong lmao")
      }
    }
  }
}


pub struct BuildConfigs {
  pub addr: SocketAddr,
  
  #[cfg(feature = "alloc")]
  pub window_name: alloc::string::String,
  
  #[cfg(feature = "alloc")]
  pub search_paths: alloc::vec::Vec<&'static str>,
  
  pub window_width: u32,
  pub window_height: u32
}