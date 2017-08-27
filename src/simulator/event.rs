
pub enum SimulatorEvent
{
    /// A processes terminated successfully
    Finished,

    /// A process terminated
    Terminated(usize),

    /// Nothing happened
    None,
}

