
use std::fmt;

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

impl fmt::Display for OpMode
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let out = match *self {
            OpMode::A  => "A",
            OpMode::B  => "B",
            OpMode::AB => "AB",
            OpMode::BA => "BA",
            OpMode::X  => "X",
            OpMode::F  => "F",
            OpMode::I  => "I",
        };

        write!(f, "{}", out)
    }
}

