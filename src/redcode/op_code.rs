
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

impl OpCode {
    /// `Seq` is identical to `Cmp`    
    #[allow(non_upper_case_globals)]
    pub const Cmp: OpCode = OpCode::Seq;   
}

