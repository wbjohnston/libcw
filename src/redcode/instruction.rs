
use redcode::{OpField, Field};

/// Redcode instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction
{
    pub op: OpField,
    pub a:  Field,
    pub b:  Field
}

impl Default for Instruction
{
    fn default() -> Self
    {
        Instruction {
            op: OpField::default(),
            a:  Field::default(),
            b:  Field::default()
        }
    }
}
