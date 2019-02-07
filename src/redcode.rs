use {
  self::{AddressingMode::*, OpCode::*, OpMode::*},
  std::fmt,
};

/// An address
pub type Address = u32;

/// An instruction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Instruction {
  pub op: OpField,
  pub a: Field,
  pub b: Field,
}

impl fmt::Display for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.op, self.a, self.b)
  }
}

impl Instruction {
  // TODO: move this into the parser
  pub fn corrected_opmode(&self) -> OpMode {
    match self.op.code {
      Dat | Nop => OpMode::F,
      // If A-mode is immediate, .AB,
      // if B-mode is immediate and A-mode isn't, .B,
      // if neither mode is immediate, .I.
      Mov | Seq | Sne | Cmp => match (self.a.mode, self.b.mode) {
        (Immediate, _) => OpMode::AB,
        (a, Immediate) if a != Immediate => OpMode::B,
        _ => OpMode::I,
      },
      // If A-mode is immediate, .AB,
      // if B-mode is immediate and A-mode isn't, .B,
      // if neither mode is immediate, .F.
      Add | Sub | Mul | Div | Mod => match (self.a.mode, self.b.mode) {
        (Immediate, _) => OpMode::AB,
        (a, Immediate) if a != Immediate => OpMode::B,
        _ => OpMode::F,
      },
      // If A-mode is immediate, .AB,
      // if it isn't, (always!) .B.
      Slt | Ldp | Stp => {
        if self.a.mode == Immediate {
          OpMode::AB
        } else {
          OpMode::B
        }
      }
      Jmp | Jmz | Jmn | Djn | Spl => OpMode::B,
    }
  }
}

impl Instruction {
  pub fn new(
    opcode: OpCode,
    opmode: OpMode,
    a_mode: AddressingMode,
    a_value: Address,
    b_mode: AddressingMode,
    b_value: Address,
  ) -> Self {
    Self {
      op: OpField {
        mode: opmode,
        code: opcode,
      },
      a: Field {
        value: a_value,
        mode: a_mode,
      },
      b: Field {
        value: b_value,
        mode: b_mode,
      },
    }
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InstructionBuilder {
  opcode: OpCode,
  opmode: OpMode,
  a_mode: AddressingMode,
  a_value: Address,
  b_mode: AddressingMode,
  b_value: Address,
}

impl InstructionBuilder {
  pub fn new(code: OpCode) -> Self {
    let default = Self::default();

    Self {
      opcode: code,
      ..default
    }
  }

  pub fn opmode(&mut self, opmode: OpMode) -> &mut Self {
    self.opmode = opmode;
    self
  }

  pub fn a_mode(&mut self, a_mode: AddressingMode) -> &mut Self {
    self.a_mode = a_mode;
    self
  }

  pub fn a_value(&mut self, a_value: Address) -> &mut Self {
    self.a_value = a_value;
    self
  }

  pub fn b_mode(&mut self, b_mode: AddressingMode) -> &mut Self {
    self.b_mode = b_mode;
    self
  }

  pub fn b_value(&mut self, b_value: Address) -> &mut Self {
    self.b_value = b_value;
    self
  }

  pub fn build(&self) -> Instruction {
    Instruction {
      op: OpField {
        code: self.opcode,
        mode: self.opmode,
      },
      a: Field {
        value: self.a_value,
        mode: self.a_mode,
      },
      b: Field {
        value: self.b_value,
        mode: self.b_mode,
      },
    }
  }
}

/// An instruction field containing the mode and opcode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpField {
  pub code: OpCode,
  pub mode: OpMode,
}

impl fmt::Display for OpField {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}.{}", self.code, self.mode)
  }
}

/// An opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
  /// Data
  Dat,
  /// Move
  Mov,
  /// Add
  Add,
  /// Sub
  Sub,
  /// Multiply
  Mul,
  /// Divide
  Div,
  /// Modulo
  Mod,
  /// Unconditional Jump
  Jmp,
  /// Jump if zero
  Jmz,
  /// Jump if not zero
  Jmn,
  /// Decrement and then Jump if not zero
  Djn,
  /// Split process
  Spl,
  /// Compare two instructions and skip next instruction if equal
  Cmp,
  /// Compare two instructions and skip next instruction if equal
  Seq,
  /// Compare two instructions and skip next instruction if not equal
  Sne,
  /// Compare two instructions and skip next instruction if less then
  Slt,
  /// Load form P-space
  Ldp,
  /// Store in P-space
  Stp,
  /// No operation
  Nop,
}

impl Default for OpCode {
  fn default() -> Self {
    OpCode::Dat
  }
}

impl fmt::Display for OpCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let s = match *self {
      Dat => "DAT",
      Mov => "MOV",
      Add => "ADD",
      Sub => "SUB",
      Mul => "MUL",
      Div => "DIV",
      Mod => "MOD",
      Jmp => "JMP",
      Jmz => "JMZ",
      Jmn => "JMN",
      Djn => "DJN",
      Spl => "SPL",
      Cmp => "CMP",
      Seq => "SEQ",
      Sne => "SNE",
      Slt => "SLT",
      Ldp => "LDP",
      Stp => "STP",
      Nop => "NOP",
    };

    write!(f, "{}", s)
  }
}

/// A opcode modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpMode {
  // A -> A
  A,
  // B -> B
  B,
  // A -> B
  AB,
  // B -> A
  BA,
  // (A, B) -> (A, B)
  F,
  // (A, B) -> (B, A)
  X,
  // All -> All
  I,
}

impl Default for OpMode {
  fn default() -> Self {
    OpMode::I
  }
}

impl fmt::Display for OpMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let s = match *self {
      A => "A",
      B => "B",
      AB => "AB",
      BA => "BA",
      F => "F",
      X => "X",
      I => "I",
    };

    write!(f, "{}", s)
  }
}

/// An instruction field
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Field {
  pub value: Address,
  pub mode: AddressingMode,
}

impl fmt::Display for Field {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}{}", self.mode, self.value)
  }
}

/// A `Field`'s adressing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
  Immediate,
  Direct,
  AIndirect(IncrementMode),
  BIndirect(IncrementMode),
}

impl AddressingMode {
  pub fn is_immediate(&self) -> bool {
    match *self {
      Immediate => true,
      _ => false,
    }
  }
}

impl fmt::Display for AddressingMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let s = match *self {
      Immediate => "#",
      Direct => "$",
      AIndirect(IncrementMode::None) => "*",
      AIndirect(IncrementMode::PostIncrement) => "}",
      AIndirect(IncrementMode::PreDecrement) => "{",
      BIndirect(IncrementMode::None) => "@",
      BIndirect(IncrementMode::PostIncrement) => ">",
      BIndirect(IncrementMode::PreDecrement) => "<",
    };

    write!(f, "{}", s)
  }
}

impl Default for AddressingMode {
  fn default() -> Self {
    AddressingMode::Direct
  }
}

/// A `AddressingMode`s increment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementMode {
  None,
  PreDecrement,
  PostIncrement,
}
