//! Your one-stop shop for everything Core Wars

use {
  self::{AddressingMode::*, Opcode::*},
  std::collections::VecDeque,
};

#[derive(Debug, Clone)]
pub struct Mars {
  memory: Vec<Instruction>,
  player_pcs: VecDeque<(usize, VecDeque<usize>)>,
}

impl Mars {
  /// Return the next program counter
  pub fn pc(&self) -> Option<usize> {
    self
      .player_pcs
      .front()
      .and_then(|(_id, queue)| queue.front().cloned())
  }

  /// Returns the current number of processes
  pub fn process_count(&self) -> usize {
    self.player_pcs.len()
  }

  /// Return the number of threads each process has along with the process id
  pub fn thread_count(&self) -> Vec<(usize, usize)> {
    self
      .player_pcs
      .iter()
      .map(|(id, process)| (*id, process.len()))
      .collect()
  }

  /// Return a view of the Mars' memory
  pub fn memory(&self) -> &[Instruction] {
    self.memory.as_slice()
  }

  /// Step forward one tick
  ///
  /// # Panics
  /// panics if there are no processes in the Mars
  pub fn step(&mut self) {
    let size = self.memory.len();
    let (id, mut queue) = self // dequeue the next process
      .player_pcs
      .pop_front()
      .expect("cannot step if no processes exist");
    let pc = queue // dequeue the next thread's program counter
      .pop_front()
      .expect("cannot execute a process with no threads");
    let instr = self.memory[pc]; // fetch instruction from memory

    // Preincrement phase
    match instr.a.mode {
      AIndirect(IncrementMode::PreDecrement) => self.memory[pc + instr.a.value].a.value -= 1,
      BIndirect(IncrementMode::PreDecrement) => self.memory[pc + instr.a.value].b.value -= 1,
      _ => {}
    }

    match instr.b.mode {
      AIndirect(IncrementMode::PreDecrement) => self.memory[pc + instr.b.value].a.value -= 1,
      BIndirect(IncrementMode::PreDecrement) => self.memory[pc + instr.b.value].b.value -= 1,
      _ => {}
    }

    // Execution phase, if a valid instruction was executed Some(offset) is
    // is returned. the offset is the amount that the program counter is
    // incremented
    let maybe_offset: Option<usize> = {
      // Calculate target addresses
      let a_target_address = pc
        + match instr.a.mode {
          Direct => instr.b.value,
          AIndirect(..) => instr.a.value + self.memory[pc + instr.a.value].a.value,
          BIndirect(..) => instr.a.value + self.memory[pc + instr.a.value].b.value,
          Immediate => 0,
        };

      let b_target_address = pc
        + match instr.b.mode {
          Direct => instr.b.value,
          AIndirect(..) => instr.b.value + self.memory[pc + instr.b.value].a.value,
          BIndirect(..) => instr.b.value + self.memory[pc + instr.b.value].b.value,
          Immediate => 0,
        };

      // This block lets us take two mutable references so we can modify either
      // the A or B target
      let (a_ptr, b_ptr) = if a_target_address < b_target_address {
        // A is lower
        let (lower, upper) = self.memory.split_at_mut(a_target_address);
        (
          &mut lower[a_target_address],
          &mut upper[b_target_address - a_target_address],
        )
      } else {
        // B is lower
        let (lower, upper) = self.memory.split_at_mut(b_target_address);
        (
          &mut upper[a_target_address - b_target_address],
          &mut lower[b_target_address],
        )
      };

      // Instruction execution phase
      use self::Opmode::*;
      match (instr.op.code, instr.op.mode) {
        // Data instructions - only kill
        (Dat, _) => None, // kill process

        // Move instructions
        (Mov, A) => {
          b_ptr.a = a_ptr.a;
          Some(1)
        }
        (Mov, B) => {
          b_ptr.b = a_ptr.b;
          Some(1)
        }
        (Mov, AB) => {
          b_ptr.b = a_ptr.a;
          Some(1)
        }
        (Mov, BA) => {
          b_ptr.a = a_ptr.b;
          Some(1)
        }
        (Mov, F) => {
          b_ptr.a = a_ptr.a;
          b_ptr.b = a_ptr.b;
          Some(1)
        }
        (Mov, X) => {
          b_ptr.b = a_ptr.a;
          b_ptr.a = a_ptr.b;
          Some(1)
        }
        (Mov, I) => {
          *b_ptr = *a_ptr;
          Some(1)
        }

        // Addition instructions
        (Add, A) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, B) => {
          b_ptr.b.value = (b_ptr.b.value + a_ptr.b.value) % size;
          Some(1)
        }
        (Add, AB) => {
          b_ptr.b.value = (b_ptr.b.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, BA) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.b.value) % size;
          Some(1)
        }
        (Add, F) | (Add, I) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.a.value) % size;
          b_ptr.b.value = (b_ptr.b.value + a_ptr.b.value) % size;
          Some(1)
        }
        (Add, X) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.b.value) % size;
          b_ptr.b.value = (b_ptr.b.value + a_ptr.a.value) % size;
          Some(1)
        }

        // Subtraction instructions
        (Sub, A) => {
          // dont collapse plz
          Some(1)
        }
        (Sub, B) => unimplemented!(),
        (Sub, AB) => unimplemented!(),
        (Sub, BA) => unimplemented!(),
        (Sub, F) => unimplemented!(),
        (Sub, X) => unimplemented!(),
        (Sub, I) => unimplemented!(),

        // Multiplication instructions
        (Mul, A) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, B) => {
          b_ptr.b.value = (b_ptr.b.value * a_ptr.b.value) % size;
          Some(1)
        }
        (Mul, AB) => {
          b_ptr.b.value = (b_ptr.b.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, BA) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.b.value) % size;
          Some(1)
        }
        (Mul, F) | (Mul, I) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.a.value) % size;
          b_ptr.b.value = (b_ptr.b.value * a_ptr.b.value) % size;
          Some(1)
        }
        (Mul, X) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.b.value) % size;
          b_ptr.b.value = (b_ptr.b.value * a_ptr.a.value) % size;
          Some(1)
        }

        // Division instructions
        (Div, A) => {
          if b_ptr.a.value != 0 {
            b_ptr.a.value = (a_ptr.a.value / b_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, B) => {
          if b_ptr.b.value != 0 {
            b_ptr.b.value = (a_ptr.b.value / b_ptr.b.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, AB) => {
          if b_ptr.a.value != 0 {
            b_ptr.b.value = (a_ptr.a.value / b_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, BA) => unimplemented!(),
        (Div, F) => unimplemented!(),
        (Div, X) => unimplemented!(),
        (Div, I) => unimplemented!(),

        // Modulo instructions
        (Mod, A) => unimplemented!(),
        (Mod, B) => unimplemented!(),
        (Mod, AB) => unimplemented!(),
        (Mod, BA) => unimplemented!(),
        (Mod, F) => unimplemented!(),
        (Mod, X) => unimplemented!(),
        (Mod, I) => unimplemented!(),

        // Jump instructions
        (Jmp, _) => Some(a_ptr.a.value),

        //
        (Jmz, A) => unimplemented!(),
        (Jmz, B) => unimplemented!(),
        (Jmz, AB) => unimplemented!(),
        (Jmz, BA) => unimplemented!(),
        (Jmz, F) => unimplemented!(),
        (Jmz, X) => unimplemented!(),
        (Jmz, I) => unimplemented!(),

        //
        (Jmn, A) => unimplemented!(),
        (Jmn, B) => unimplemented!(),
        (Jmn, AB) => unimplemented!(),
        (Jmn, BA) => unimplemented!(),
        (Jmn, F) => unimplemented!(),
        (Jmn, X) => unimplemented!(),
        (Jmn, I) => unimplemented!(),

        //
        (Djn, A) => unimplemented!(),
        (Djn, B) => unimplemented!(),
        (Djn, AB) => unimplemented!(),
        (Djn, BA) => unimplemented!(),
        (Djn, F) => unimplemented!(),
        (Djn, X) => unimplemented!(),
        (Djn, I) => unimplemented!(),

        // Split instructions
        (Spl, _) => {
          // Start new thread by queuing new program counter
          queue.push_back(pc + a_ptr.a.value);
          Some(1)
        }

        //
        (Seq, A) | (Cmp, A) => unimplemented!(),
        (Seq, B) | (Cmp, B) => unimplemented!(),
        (Seq, AB) | (Cmp, AB) => unimplemented!(),
        (Seq, BA) | (Cmp, BA) => unimplemented!(),
        (Seq, F) | (Cmp, F) => unimplemented!(),
        (Seq, X) | (Cmp, X) => unimplemented!(),
        (Seq, I) | (Cmp, I) => unimplemented!(),

        (Slt, A) => unimplemented!(),
        (Slt, B) => unimplemented!(),
        (Slt, AB) => unimplemented!(),
        (Slt, BA) => unimplemented!(),
        (Slt, F) => unimplemented!(),
        (Slt, X) => unimplemented!(),
        (Slt, I) => unimplemented!(),

        (Sne, A) => unimplemented!(),
        (Sne, B) => unimplemented!(),
        (Sne, AB) => unimplemented!(),
        (Sne, BA) => unimplemented!(),
        (Sne, F) => unimplemented!(),
        (Sne, X) => unimplemented!(),
        (Sne, I) => unimplemented!(),

        (Ldp, A) => unimplemented!(),
        (Ldp, B) => unimplemented!(),
        (Ldp, AB) => unimplemented!(),
        (Ldp, BA) => unimplemented!(),
        (Ldp, F) => unimplemented!(),
        (Ldp, X) => unimplemented!(),
        (Ldp, I) => unimplemented!(),

        (Stp, A) => unimplemented!(),
        (Stp, B) => unimplemented!(),
        (Stp, AB) => unimplemented!(),
        (Stp, BA) => unimplemented!(),
        (Stp, F) => unimplemented!(),
        (Stp, X) => unimplemented!(),
        (Stp, I) => unimplemented!(),

        (Nop, _) => Some(1),
      }
    };

    // post increment
    match instr.a.mode {
      AIndirect(IncrementMode::PostIncrement) => self.memory[pc + instr.a.value].a.value += 1,
      BIndirect(IncrementMode::PostIncrement) => self.memory[pc + instr.a.value].b.value += 1,
      _ => {}
    }

    match instr.b.mode {
      AIndirect(IncrementMode::PostIncrement) => self.memory[pc + instr.b.value].a.value += 1,
      BIndirect(IncrementMode::PostIncrement) => self.memory[pc + instr.b.value].b.value += 1,
      _ => {}
    }

    // requeue the program counter if the thread wasn't killed
    if let Some(offset) = maybe_offset {
      queue.push_back(pc + offset);
    }

    // requeue the process if there are still threads
    if !queue.is_empty() {
      self.player_pcs.push_back((id, queue));
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarsBuilder {
  size: usize,
}

impl MarsBuilder {
  /// Create a new `MarsBuilder` with required parameters
  pub fn new(size: usize) -> Self {
    // TODO: implement me
    unimplemented!()
  }

  /// Build a `Mars`
  pub fn build() -> Mars {
    /// TODO: implement me
    unimplemented!()
  }
}

/// An instruction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Instruction {
  pub op: OpField,
  pub a: Field,
  pub b: Field,
}

impl Instruction {
  pub fn corrected_opmode(&self) -> Opmode {
    match self.op.code {
      Dat | Nop => Opmode::F,
      // If A-mode is immediate, .AB,
      // if B-mode is immediate and A-mode isn't, .B,
      // if neither mode is immediate, .I.
      Mov | Seq | Sne | Cmp => match (self.a.mode, self.b.mode) {
        (Immediate, _) => Opmode::AB,
        (a, Immediate) if a != Immediate => Opmode::B,
        _ => Opmode::I,
      },
      // If A-mode is immediate, .AB,
      // if B-mode is immediate and A-mode isn't, .B,
      // if neither mode is immediate, .F.
      Add | Sub | Mul | Div | Mod => match (self.a.mode, self.b.mode) {
        (Immediate, _) => Opmode::AB,
        (a, Immediate) if a != Immediate => Opmode::B,
        _ => Opmode::F,
      },
      // If A-mode is immediate, .AB,
      // if it isn't, (always!) .B.
      Slt | Ldp | Stp => {
        if self.a.mode == Immediate {
          Opmode::AB
        } else {
          Opmode::B
        }
      }
      Jmp | Jmz | Jmn | Djn | Spl => Opmode::B,
    }
  }
}

/// An instruction field containing the mode and opcode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpField {
  pub code: Opcode,
  pub mode: Opmode,
}

/// An opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
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

impl Default for Opcode {
  fn default() -> Self {
    Opcode::Dat
  }
}

/// A opcode modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opmode {
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

impl Default for Opmode {
  fn default() -> Self {
    Opmode::I
  }
}

/// An instruction field
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Field {
  pub value: usize,
  pub mode: AddressingMode,
}

/// A `Field`'s adressing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
  Immediate,
  Direct,
  AIndirect(IncrementMode),
  BIndirect(IncrementMode),
}

impl Default for AddressingMode {
  fn default() -> Self {
    AddressingMode::Immediate
  }
}

/// A `AddressingMode`s increment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementMode {
  None,
  PreDecrement,
  PostIncrement,
}
