
use std::fmt;

use redcode::types::{Modifier, Value, OpCode, AddressingMode};
use redcode::traits;

/// Redcode instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction
{
    op:       OpCode,
    modifier: Modifier,
    a:        Value,
    a_mode:   AddressingMode, 
    b:        Value,
    b_mode:   AddressingMode
}

impl Instruction
{
    /// Create a new instruction
    #[inline]
    pub fn new(
        op: OpCode,
        modifier: Modifier,
        a: Value,
        a_mode: AddressingMode,
        b: Value,
        b_mode: AddressingMode,
        ) -> Self
    {
        Self {op, modifier, a, a_mode, b, b_mode}
    }
}

impl Default for Instruction
{
    fn default() -> Instruction
    {
        Instruction {
            op: OpCode::Dat,
            modifier: Modifier::I,
            a: Value::default(),
            a_mode: AddressingMode::Direct,
            b: Value::default(),
            b_mode: AddressingMode::Direct,
        }
    }
}

impl traits::Instruction for Instruction
{
    #[inline]
    fn op(&self) -> OpCode
    {
        self.op
    }

    #[inline]
    fn set_op(&mut self, v: OpCode)
    {
        self.op = v;
    }

    #[inline]
    fn modifier(&self) -> Modifier
    {
        self.modifier
    }

    #[inline]
    fn set_modifier(&mut self, v: Modifier)
    {
        self.modifier = v;
    }

    #[inline]
    fn a(&self) -> Value
    {
        self.a
    }

    #[inline]
    fn set_a(&mut self, v: Value)
    {
        self.a = v;
    }

    #[inline]
    fn a_mode(&self) -> AddressingMode
    {
        self.a_mode
    }

    #[inline]
    fn set_a_mode(&mut self, v: AddressingMode)
    {
        self.a_mode = v;
    }

    #[inline]
    fn b(&self) -> Value
    {
        self.b
    }

    #[inline]
    fn set_b(&mut self, v: Value)
    {
        self.b = v;
    }

    #[inline]
    fn b_mode(&self) -> AddressingMode
    {
        self.b_mode
    }

    #[inline]
    fn set_b_mode(&mut self, v: AddressingMode)
    {
        self.b_mode = v;
    }
}

impl fmt::Display for Instruction
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(
            f,
            "{}.{} {}{} {}{}",
            self.op,
            self.modifier,
            self.a,
            self.a_mode,
            self.b,
            self.b_mode
            )
    } 
}

