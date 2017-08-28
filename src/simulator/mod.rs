//! Corewars simulator

mod simulator_builder;
pub use self::simulator_builder::SimulatorBuilder;

mod error;
pub use self::error::SimulatorError;

mod event;
pub use self::event::SimulatorEvent;

mod simulator;
pub use self::simulator::Simulator;

