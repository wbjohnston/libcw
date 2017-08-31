
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

