
use redcode::{OpCode, OpMode};

/// Field Containg the `OpCode` and `OpMode`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OpField
{
    pub code: OpCode,
    pub mode: OpMode
}

impl Default for OpField
{
    fn default() -> Self
    {
        OpField {
            code: OpCode::Dat,
            mode: OpMode::F
        }
    }
}

