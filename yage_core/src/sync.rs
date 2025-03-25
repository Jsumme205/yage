use core::{pin::Pin, task::Context};

use yage_executor::{Executor, NotThreadSafe};
use yage_util::{atomic::AtomicMut, list::LinkedList};

use crate::component::{
    component_handle::ComponentHandle,
    sync::{AsyncComponent, AsyncRenderContext},
    RenderContext,
};

type AsyncComponentHandle<S> = ComponentHandle<S, dyn AsyncComponent<State = S>>;

pub(crate) enum Async<S> {
    Enabled {
        executor: Executor<NotThreadSafe>,
        components: LinkedList<AsyncComponentHandle<S>>,
    },
    Disabled,
}

impl<S> Async<S> {
    pub const fn disabled() -> Self {
        Self::Disabled
    }

    pub fn enabled() -> Self {
        Self::Enabled {
            executor: Executor::new_unsync(),
            components: LinkedList::new(),
        }
    }

    pub(crate) async fn poll(
        &mut self,
        ctx: &mut RenderContext<S>,
        cx: &mut Context<'_>,
    ) -> crate::Result<()> {
        todo!()
    }
}
