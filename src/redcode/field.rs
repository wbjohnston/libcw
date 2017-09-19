//! Redcode `Instruction` field

use std::fmt;

use redcode::{Value, AddressingMode};

/// Field containing addressing mode and offset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Field
{
    pub value:  Value,
    pub mode:   AddressingMode
}

impl fmt::Display for Field
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}{}", self.mode, self.value)
    }
}

impl Default for Field
{
    fn default() -> Self
    {
        Field {
            value: 0,
            mode:   AddressingMode::Direct
        }
    }
}

