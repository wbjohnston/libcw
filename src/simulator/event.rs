//! Events that can happen during a running simulation

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulatorEvent
{
    /// All processes terminated successfully
    Finished,

    /// A process terminated
    Terminated(usize),

    /// Nothing happened
    None,
}

