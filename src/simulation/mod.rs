//! Datastructures and functions for building and simulating a redcode core

mod mars;
pub use self::mars::{
    Mars,
    MarsEvent,
    MarsResult
};

mod builder;
pub use self::builder::{
    MarsBuilder,
    BuilderError
};


