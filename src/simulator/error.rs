//! Simulator errors

/// Simulator errors
#[derive(Debug, PartialEq, Eq)]
pub enum SimulatorError
{
    NotEnoughMemory,

    PrematureTermination
}

