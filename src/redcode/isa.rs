/// Red Code instruction set architecture

/// Operations that a redcode processor can perform
///
/// TODO: longform explanation
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode
{
    /// Data instruction, kills process queue on execution
    Dat,

    /// Move instruction
    Mov,

    /// Add instruction
    Add,

    /// Subtract instruction
    Sub,

    /// Multiply instruction
    Mul,

    /// Divide instruction
    Div,

    /// Modulo instruction
    Mod,

    /// Jump instruction
    Jmp,

    /// Jump if zero
    Jmz,

    /// Jump if not zero
    Jmn,
    
    /// TODO
    Djn,

    /// Split instruction
    Spl,

    /// Compare instruction
    Cmp,

    /// Set if equal instruction
    Seq,
    
    /// Set if not equal instruction
    Sne,

    /// Set if less than instruction
    Slt,

    /// TODO
    Ldp,

    /// TODO
    Stp,

    /// No operation instruction
    Nop
}

/// Controls modes for what components of an instruction and OPCODE will 
/// operate on
///
/// TODO: examples
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
///
/// TODO: examples
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AddressingMode
{
    /// TODO
    Direct,

    /// TODO
    AIndirect,

    /// TODO
    BIndirect,

    /// TODO
    AIndirectPreDecrement,

    /// TODO
    BIndirectPreDecrement,

    /// TODO
    AIndirectPostIncrement,

    /// TODO
    BIndirectPostIncrement,
}

/// Field containg the opcode and opmode
#[derive(Debug, Copy, Clone)]
pub struct OpField
{
    pub op: OpCode,
    pub mode:   OpMode
}

/// TODO: docs
#[derive(Debug, Copy, Clone)]
pub struct Field
{
    pub offset:          isize,
    pub mode: AddressingMode,
}

/// Recode instruction
#[derive(Debug, Copy, Clone)]
pub struct Instruction
{
    pub op: OpField,
    pub a:  Field,
    pub b:  Field
}

