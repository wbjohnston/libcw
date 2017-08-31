//! Datastructures and functions for building and simulating a redcode core

mod core;
pub use self::core::{
    Core,
    CoreBuilder,
    BuilderError,
};

mod error;
pub use self::error::Error;

mod event;
pub use self::event::Event;

