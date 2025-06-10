

pub struct Cleanup<C, M, Eq, S> {
  pub(crate) cleanup_plugin: C,
  pub(crate) main_plugin: M, 
  pub(crate) event_queue: Eq,
  pub(crate) state: S,
}