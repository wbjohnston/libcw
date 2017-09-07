
use std::fmt;

use redcode::{OpField, Field};

/// Redcode instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction
{
    pub op: OpField,
    pub a:  Field,
    pub b:  Field
}

impl fmt::Display for Instruction
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{} {} {}", self.op, self.a, self.b)
    }
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
