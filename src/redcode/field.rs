//! Redcode `Instruction` field

use redcode::{Offset, AddressingMode};

/// Field containing addressing mode and offset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Field
{
    pub offset: Offset,
    pub mode:   AddressingMode
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

