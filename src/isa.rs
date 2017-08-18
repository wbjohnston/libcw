/// Red Code instruction set architecture

/// Operations that a redcode processor can perform
///
/// TODO: longform explanation
#[allow(dead_code)]
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

    /// TODO
    Jmz,

    /// TODO
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

