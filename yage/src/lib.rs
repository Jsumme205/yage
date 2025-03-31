pub use yage_core::{ErrorKind, Result, errors::Error};

pub mod system {
    pub use yage_system::{FailableSystem, RunSystem, System, Systems};
}

pub mod window {
    pub use yage_core::window::{Metrics, Window};
}

pub mod assets;
pub mod engine;
