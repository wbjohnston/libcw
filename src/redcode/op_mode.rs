
/// Controls modes for what components of an instruction and OPCODE will 
/// operate on
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpMode
{
    /// A-field to A-field
    A,

    /// B-field to B-field
    B,

    /// A field to B field
    AB,

    /// B-field to A-field
    BA,

    /// A-field to B-field AND B-field to A-field
    X,

    /// A-field to A-field AND B-field to B-field
    F,

    /// Whole instruction
    I,
}
