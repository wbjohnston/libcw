//! Redcode `Instruction` field

use std::fmt;

use redcode::{Offset, AddressingMode};

/// Field containing addressing mode and offset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Field
{
    pub offset: Offset,
    pub mode:   AddressingMode
}

impl fmt::Display for Field
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}{}", self.mode, self.offset)
    }
}

impl Default for Field
{
    fn default() -> Self
    {
        Field {
            offset: 0,
            mode:   AddressingMode::Direct
        }
    }
}

