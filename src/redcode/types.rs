//! Supporting redcode types

use std::fmt;

/// Address in a core
pub type Address = u32;

/// `Field` Value 
pub type Value = i16;

/// Process ID
pub type Pid = u16;

/// P-space PIN
pub type Pin = Pid;

/// Operations that a redcode processor can perform
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode
{
    /// Data, kills thread on execution
    Dat,

    /// Move (copy)
    Mov,

    /// Add
    Add,

    /// Subtract
    Sub,

    /// Multiply
    Mul,

    /// Divide
    Div,

    /// Modulo
    Mod,

    /// Jump
    Jmp,

    /// Jump if zero
    Jmz,

    /// Jump if not zero
    Jmn,

    /// Decrement by one and Jump if not zero
    Djn,

    /// Create a new execution thread at target address
    Spl,

    /// Compare
    Seq,

    /// Skip if not equal
    Sne,

    /// Skip if less than
    Slt,

    /// Load from P-space
    Ldp,

    /// Save to P-space
    Stp,

    /// No operation
    Nop
}

impl fmt::Display for OpCode
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let out_str = match *self {
            OpCode::Dat => "dat",
            OpCode::Mov => "mov",
            OpCode::Add => "add",
            OpCode::Sub => "sub",
            OpCode::Mul => "mul",
            OpCode::Div => "div",
            OpCode::Mod => "mod",
            OpCode::Jmp => "jmp",
            OpCode::Jmz => "jmz",
            OpCode::Jmn => "jmn",
            OpCode::Djn => "djn",
            OpCode::Spl => "spl",
            OpCode::Seq => "seq",
            OpCode::Sne => "sne",
            OpCode::Slt => "slt",
            OpCode::Ldp => "ldp",
            OpCode::Stp => "stp",
            OpCode::Nop => "stp",
        };

        write!(f, "{}", out_str)
    }
}

/// Controls modes for what components of an instruction and OPCODE will
/// operate on
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Modifier
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

impl fmt::Display for Modifier
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let out_str = match *self {
            Modifier::A  => "A",
            Modifier::B  => "B",
            Modifier::AB => "AB",
            Modifier::BA => "BA",
            Modifier::X  => "X",
            Modifier::F  => "F",
            Modifier::I  => "I",
        };

        write!(f, "{}", out_str)
    }
}

/// Field Addressing mode: controls how the `offset` behaves
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AddressingMode
{
    /// Literal value e.g "2"
    ///
    /// Denoted by: `#`
    Immediate,

    /// Direct pointer to another instruction
    ///
    /// Denoted by: `$`
    Direct,

    /// Indirect addressing by target's A field
    ///
    /// Denoted by: `*`
    AIndirect,

    /// Indirect addressing by target's B field
    ///
    /// Denoted by: `@`
    BIndirect,

    /// Indirect addressing by target's A field, target instructions A field is
    /// decremented before calculating the target address
    ///
    /// Denoted by: `{`
    AIndirectPreDecrement,

    /// Indirect addressing by target's A field, target instructions B field is
    /// decremented before calculating the target address
    ///
    /// Denoted by: `<`
    BIndirectPreDecrement,

    /// Indirect addressing by target's A field, target instructions B field is
    /// incremented after calculating the target address
    ///
    /// Denoted by: `}`
    AIndirectPostIncrement,

    /// Indirect addressing by target's B field, target instructions B field is
    /// incremented after calculating the target address
    ///
    /// Denoted by: `>`
    BIndirectPostIncrement,
}

impl fmt::Display for AddressingMode
{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let out_str = match *self {
            AddressingMode::Immediate              => "#",
            AddressingMode::Direct                 => "$",
            AddressingMode::AIndirect              => "*",
            AddressingMode::BIndirect              => "@",
            AddressingMode::BIndirectPreDecrement  => "<",
            AddressingMode::BIndirectPostIncrement => ">",
            AddressingMode::AIndirectPreDecrement  => "{",
            AddressingMode::AIndirectPostIncrement => "}",
        };

        write!(f, "{}", out_str)
    }
}

