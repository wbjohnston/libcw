//! Datastructures and functions for building and simulating a redcode core

mod mars;
pub use self::mars::{
    Mars,
    LoadResult,
    LoadError,
    SimulationResult,
    SimulationEvent,
    SimulationError
};

mod builder;
pub use self::builder::{
    MarsBuilder,
    BuilderError
};


