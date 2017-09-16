
 use std::fmt;

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
        let out = match *self {
            AddressingMode::Immediate              => "#",
            AddressingMode::Direct                 => "$",
            AddressingMode::AIndirect              => "*",
            AddressingMode::AIndirectPreDecrement  => "{",
            AddressingMode::AIndirectPostIncrement => "}",
            AddressingMode::BIndirect              => "@",
            AddressingMode::BIndirectPreDecrement  => "<",
            AddressingMode::BIndirectPostIncrement => ">"
        };

        write!(f, "{}", out)
    }
}

