//! Traits for describing redcode instructions

use super::types::{Value, OpCode, Modifier, AddressingMode};

/// A trait indicating that a the implementing type can be interpreted as an 
/// redcode instruction
pub trait Instruction: Default + Clone
{
    /// Get the opcode
    fn op(&self) -> OpCode;

    /// Set the instruction opcode
    fn set_op(&mut self, v: OpCode);

    /// Get Operation Mode
    fn modifier(&self) -> Modifier;

    /// Set operation mode
    fn set_modifier(&mut self, v: Modifier);

    /// Get the value of the A field
    fn a(&self) -> Value;

    /// Set the value of the A field
    fn set_a(&mut self, v: Value);

    /// Get the addressing mode of the A Field
    fn a_mode(&self) -> AddressingMode;

    /// Set the addressing mode of the A field
    fn set_a_mode(&mut self, v: AddressingMode);

    /// Get value of the B field
    fn b(&self) -> Value;

    /// Set the value of the B field
    fn set_b(&mut self, v: Value);

    /// Get the addressing mode of the B field
    fn b_mode(&self) -> AddressingMode;

    /// Set the AddressingMode of the B field
    fn set_b_mode(&mut self, v: AddressingMode);
}

