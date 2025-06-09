use std::any::Any;
use std::sync::Arc;

use super::event::Event;
use super::{ObjectId, QueuedWayland};
use std::os::fd::OwnedFd;

pub trait ObjectData: Any {
    fn on_event(
        self: Arc<Self>,
        handle: &QueuedWayland,
        message: Event<ObjectId, OwnedFd>,
    ) -> Option<Arc<dyn ObjectData>>;

    fn on_destruction(&self, id: ObjectId);

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dyn ObjectData").finish_non_exhaustive()
    }
}

impl dyn ObjectData {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct DummyObjectData;

impl ObjectData for DummyObjectData {
    fn on_event(
        self: Arc<Self>,
        _handle: &QueuedWayland,
        _message: Event<ObjectId, OwnedFd>,
    ) -> Option<Arc<dyn ObjectData>> {
        Some(self)
    }

    fn on_destruction(&self, _id: ObjectId) {}
}
