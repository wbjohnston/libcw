//! Datastructures and functions for building and simulating a redcode core

mod core;
pub use self::core::{
    Core,
    CoreEvent,
    CoreResult
};

mod builder;
pub use self::builder::{
    CoreBuilder,
    BuilderError
};


