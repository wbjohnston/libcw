
use std::fmt;

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

impl fmt::Display for OpCode
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let out = match *self {
            OpCode::Dat => "DAT",
            OpCode::Mov => "MOV",
            OpCode::Add => "ADD",
            OpCode::Sub => "SUB",
            OpCode::Mul => "MUL",
            OpCode::Div => "DIV",
            OpCode::Mod => "MOD",
            OpCode::Jmp => "JMP",
            OpCode::Jmz => "JMZ",
            OpCode::Jmn => "JMN",
            OpCode::Djn => "DJN",
            OpCode::Spl => "SPL",
            OpCode::Seq => "SEQ",
            OpCode::Sne => "SNE",
            OpCode::Slt => "SLT",
            OpCode::Ldp => "LDP",
            OpCode::Stp => "STP",
            OpCode::Nop => "NOP"
        };

        write!(f, "{}", out)
    }
}

