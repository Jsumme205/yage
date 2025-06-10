use super::cleanup::Cleanup;
use super::TrackedPlugin;
use crate::machine_cog::{Cog, MachineInput};
use crate::plugin::Plugin;
use crate::renderer::{MakeRenderer, Renderer};
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait EventQueue<S> {
    type Handle<State>;
    type ReadGuard;
    type Error;

    fn make_handle(&self) -> Self::Handle<S>;

    fn poll_dispatch(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        state: &mut S,
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&Self>, cx: &mut Context<'_>) -> Poll<Result<usize, Self::Error>>;

    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::ReadGuard>>;
}

pub struct MainLoopContext<'a, S, Eq: EventQueue<S>, R> {
    state: &'a mut S,
    event_queue: <Eq as EventQueue<S>>::Handle<S>,
    renderer: &'a mut R,
    _marker: core::marker::PhantomData<&'a mut Eq>,
}

pub struct MainLoop<M, S, Eq, R> {
    pub(super) main_loop: Option<M>,
    pub(super) state: Option<S>,
    pub(super) event_queue: Option<Eq>,
    pub(super) renderer: Option<R>,
    main_state: State,
}

#[derive(Clone, Copy)]
enum State {
    Ready,
    Transform,
    What,
}

struct MainLoopProjection<'__pin, M, S, Eq, R> {
    main_loop: Pin<&'__pin mut Option<M>>,
    state: &'__pin mut Option<S>,
    event_queue: Pin<&'__pin mut Option<Eq>>,
    renderer: &'__pin mut Option<R>,
    main_state: &'__pin mut State,
}

impl<M, S, Eq, R> MainLoop<M, S, Eq, R> {
    pub fn new(main_loop: M, state: S, event_queue: Eq) -> Self {
        Self {
            main_loop: Some(main_loop),
            state: Some(state),
            event_queue: Some(event_queue),
            renderer: None,
            main_state: State::Ready,
        }
    }

    fn project<'__pin>(self: Pin<&'__pin mut Self>) -> MainLoopProjection<'__pin, M, S, Eq, R> {
        let Self {
            main_loop,
            state,
            event_queue,
            renderer,
            main_state,
        } = unsafe { self.get_unchecked_mut() };
        unsafe {
            MainLoopProjection {
                main_loop: Pin::new_unchecked(main_loop),
                state,
                event_queue: Pin::new_unchecked(event_queue),
                renderer,
                main_state,
            }
        }
    }
}

crate::seal!(MainLoop<M, S, Eq, R>);

pub enum Error<ME, RE, EE> {
    MainLoopError(ME),
    RenderError(RE),
    EventQueueError(EE),
}

pub enum MainError<ME, EE> {
    LoopUserError(ME),
    EventLoopError(EE),
}

impl<M, S, Eq, R, C, ME, RE, EE> Cog<(C,)> for MainLoop<M, S, Eq, R>
where
    M: for<'a, 'b> Plugin<
        &'b mut MainLoopContext<'a, S, Eq, R>,
        Output = C,
        Error = MainError<ME, EE>,
    >,
    R: Renderer<Error = RE> + MakeRenderer,
    Eq: EventQueue<S, Error = RE>,
{
    type Error = Error<ME, RE, EE>;
    type Input = (u32, u32);
    type Output<N: crate::machine_cog::TupleHelper> = Cleanup<N::E1, M, Eq, S>;

    fn poll_transform(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &mut MachineInput<'_, Self::Input>,
        _: crate::machine_cog::OnlyCalledByThisCrate,
    ) -> Poll<Result<Self::Output<(C,)>, Self::Error>> {
        let MainLoopProjection {
            mut main_loop,
            state,
            mut event_queue,
            renderer,
            main_state,
        } = self.project();
        let (width, height) = *input.input;
        loop {
            match main_state {
                State::Ready => {
                    let handle = event_queue
                        .as_mut()
                        .as_pin_mut()
                        .unwrap()
                        .into_ref()
                        .make_handle();
                    let renderer = match R::new(width, height) {
                        Ok(r) => renderer.get_or_insert(r),
                        Err(error) => {
                            return Poll::Ready(Err(Error::RenderError(error)));
                        }
                    };
                    let mut context: MainLoopContext<'_, _, Eq, _> = MainLoopContext {
                        state: state.as_mut().unwrap(),
                        event_queue: handle,
                        renderer,
                        _marker: core::marker::PhantomData,
                    };
                    match main_loop
                        .as_mut()
                        .as_pin_mut()
                        .unwrap()
                        .poll_ready(cx, &mut context)
                    {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(())) => {
                            *main_state = State::Transform;
                        }
                        Poll::Ready(Err(err)) => match err {
                            MainError::EventLoopError(err) => {
                                return Poll::Ready(Err(Error::EventQueueError(err)))
                            }
                            MainError::LoopUserError(err) => {
                                return Poll::Ready(Err(Error::MainLoopError(err)))
                            }
                        },
                    }
                }
                State::Transform => {
                    match main_loop.as_mut().as_pin_mut().unwrap().poll_transform(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Ok(cleanup_plugin)) => {
                            *main_state = State::What;
                            unsafe {
                                return Poll::Ready(Ok(Cleanup {
                                    cleanup_plugin,
                                    main_plugin: main_loop
                                        .get_unchecked_mut()
                                        .take()
                                        .expect("this should still be here"),
                                    event_queue: event_queue
                                        .get_unchecked_mut()
                                        .take()
                                        .expect("this should still be here"),
                                    state: state.take().expect("this should still be here"),
                                }));
                            }
                        }
                        Poll::Ready(Err(err)) => match err {
                            MainError::EventLoopError(err) => {
                                return Poll::Ready(Err(Error::EventQueueError(err)))
                            }
                            MainError::LoopUserError(err) => {
                                return Poll::Ready(Err(Error::MainLoopError(err)))
                            }
                        },
                    }
                }
                State::What => panic!("what???? lmao"),
            }
        }
    }
}

