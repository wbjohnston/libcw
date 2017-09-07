
use std::fmt;

use redcode::{OpCode, OpMode};

/// Field Containg the `OpCode` and `OpMode`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OpField
{
    pub code: OpCode,
    pub mode: OpMode
}

impl fmt::Display for OpField
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}.{}", self.code, self.mode)
    }
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

