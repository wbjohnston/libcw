//! Events that can happen during a running simulation

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event
{
    /// All processes terminated successfully
    Finished,

    /// Game ended in a tie
    Tied,

    /// Process split inner contains address of new pc
    Split(usize),

    /// A process terminated
    Terminated(usize),

    /// Nothing happened
    None,
}

