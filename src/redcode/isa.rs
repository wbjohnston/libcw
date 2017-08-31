/// Red Code instruction set architecture

use std::default::Default;

/// Alias for a program, which is just a list of instructions
pub type Program = Vec<Instruction>;

pub type Address = usize;

pub type Offset = isize;

/// Operations that a redcode processor can perform
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode
{
    /// Data, kills process queue on execution
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

    /// Compare (same as `Seq`)
    Cmp,

    /// Skip if equal
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

/// Controls modes for what components of an instruction and OPCODE will 
/// operate on
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpMode
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

/// Field Containg the `OpCode` and `OpMode`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OpField
{
    pub code: OpCode,
    pub mode: OpMode
}

/// Field containing addressing mode and offset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Field
{
    pub offset: Offset,
    pub mode:   AddressingMode
}

/// Redcode instruction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction
{
    // FIXME: I don't like these public fields
    pub op: OpField,
    pub a:  Field,
    pub b:  Field
}

impl Default for Instruction
{
    fn default() -> Self
    {
        Instruction {
            op: OpField {
                code: OpCode::Dat,
                mode: OpMode::F
            },
            a: Field {
                offset: 0,
                mode: AddressingMode::Direct
            },
            b: Field {
                offset: 0,
                mode: AddressingMode::Direct
            }
        }
    }
}

