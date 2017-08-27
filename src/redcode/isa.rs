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

/// TODO: docs
#[derive(Debug, Copy, Clone)]
pub enum OpMode
{
    A,
    B,
    AB,
    BA,
    X,
    I,
    F
}

/// TODO: docs
#[derive(Debug, Copy, Clone)]
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

/// TODO: docs
#[derive(Debug, Copy, Clone)]
pub struct OpCodeField
{
    pub opcode: OpCode,
    pub mode:   OpMode
}

/// TODO: docs
#[derive(Debug, Copy, Clone)]
pub struct InstructionField
{
    pub offset:          isize,
    pub addressing_mode: AddressingMode,
}

/// Recode instruction
#[derive(Debug, Copy, Clone)]
pub struct Instruction
{
    pub op: OpCodeField,
    pub a:  InstructionField,
    pub b:  InstructionField
}



