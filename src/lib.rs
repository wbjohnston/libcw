//! Your one-stop shop for everything Core Wars

use {
  self::{AddressingMode::*, Opcode::*},
  std::collections::VecDeque,
};

pub type Pid = usize;
pub type Address = usize;

#[derive(Debug, Clone)]
pub struct Mars {
  memory: Vec<Instruction>,
  p_space_size: usize,
  proceses: VecDeque<(Pid, Vec<Instruction>, VecDeque<Address>)>,
}

impl Mars {
  /// Return the next program counter
  pub fn pc(&self) -> Option<Address> {
    self
      .proceses
      .front()
      .and_then(|(_, _, process)| process.front().cloned())
  }

  /// Returns the current number of processes
  pub fn process_count(&self) -> usize {
    self.proceses.len()
  }

  /// Return the number of threads each process has along with the process id
  pub fn thread_count(&self) -> Vec<(usize, usize)> {
    self
      .proceses
      .iter()
      .map(|(id, _, threads)| (*id, threads.len()))
      .collect()
  }

  /// Return a view of the Mars' memory
  pub fn memory(&self) -> &[Instruction] {
    self.memory.as_slice()
  }

  pub fn load_program(&mut self, program: &[Instruction], address: Address) -> Pid {
    let size = self.memory.len();
    let address = address % size; // normalize address

    // normalize values
    // TODO: no idea why this needs to be mutable
    let program = program
      .iter()
      .map(|i| Instruction {
        a: Field {
          value: i.a.value % size,
          ..i.a
        },
        b: Field {
          value: i.b.value % size,
          ..i.b
        },
        ..*i
      })
      .collect::<Vec<_>>(); // FIXME: bad copy

    for i in 0..program.len() {
      self.memory[(address + i) % size] = program[i];
    }

    let mut threads = VecDeque::new();
    threads.push_back(address);

    let pspace = vec![Instruction::default(); self.p_space_size];
    let pid = self.proceses.len();

    self.proceses.push_back((pid, pspace, threads));
    pid
  }

  /// Step forward one tick
  ///
  /// # Panics
  /// panics if there are no processes in the Mars
  pub fn step(&mut self) {
    assert!(self.proceses.len() > 0);
    let size = self.memory.len();
    let (id, mut pspace, mut threads) = self // dequeue the next process
      .proceses
      .pop_front()
      .expect("cannot step if no processes exist");
    let pc = threads // dequeue the next thread's program counter
      .pop_front()
      .expect("cannot execute a process with no threads");
    let instr = self.memory[pc]; // fetch instruction from memory

    let a_target_address = pc
      + match instr.a.mode {
        Direct => instr.a.value,
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
    let maybe_offset = {
      let a_ptr = self.memory[a_target_address];
      let b_ptr = &mut self.memory[b_target_address];

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
          *b_ptr = a_ptr;
          Some(1)
        }

        // Addition instructions
        (Add, A) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, B) => {
          b_ptr.b.value = (b_ptr.b.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, AB) => {
          b_ptr.b.value = (b_ptr.a.value + a_ptr.b.value) % size;
          Some(1)
        }
        (Add, BA) => {
          b_ptr.a.value = (b_ptr.b.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, X) => {
          b_ptr.b.value = (b_ptr.a.value + a_ptr.b.value) % size;
          b_ptr.a.value = (b_ptr.b.value + a_ptr.a.value) % size;
          Some(1)
        }
        (Add, I) | (Add, F) => {
          b_ptr.a.value = (b_ptr.a.value + a_ptr.a.value) % size;
          b_ptr.b.value = (b_ptr.b.value + a_ptr.b.value) % size;
          Some(1)
        }

        // Subtraction instructions
        (Sub, A) => {
          b_ptr.a.value = (b_ptr.a.value + size - a_ptr.a.value) % size;
          Some(1)
        }
        (Sub, B) => {
          b_ptr.b.value = (b_ptr.b.value + size - a_ptr.a.value) % size;
          Some(1)
        }
        (Sub, AB) => {
          b_ptr.b.value = (b_ptr.a.value + size - a_ptr.b.value) % size;
          Some(1)
        }
        (Sub, BA) => {
          b_ptr.a.value = (b_ptr.b.value + size - a_ptr.a.value) % size;
          Some(1)
        }
        (Sub, X) => {
          b_ptr.b.value = (b_ptr.a.value + size - a_ptr.b.value) % size;
          b_ptr.a.value = (b_ptr.b.value + size - a_ptr.a.value) % size;
          Some(1)
        }
        (Sub, I) | (Sub, F) => {
          b_ptr.a.value = (b_ptr.a.value + size - a_ptr.a.value) % size;
          b_ptr.b.value = (b_ptr.b.value + size - a_ptr.b.value) % size;
          Some(1)
        }

        // Multiplication instructions
        (Mul, A) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, B) => {
          b_ptr.b.value = (b_ptr.b.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, AB) => {
          b_ptr.b.value = (b_ptr.a.value * a_ptr.b.value) % size;
          Some(1)
        }
        (Mul, BA) => {
          b_ptr.a.value = (b_ptr.b.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, X) => {
          b_ptr.b.value = (b_ptr.a.value * a_ptr.b.value) % size;
          b_ptr.a.value = (b_ptr.b.value * a_ptr.a.value) % size;
          Some(1)
        }
        (Mul, I) | (Mul, F) => {
          b_ptr.a.value = (b_ptr.a.value * a_ptr.a.value) % size;
          b_ptr.b.value = (b_ptr.b.value * a_ptr.b.value) % size;
          Some(1)
        }

        // Division instructions
        (Div, A) => {
          if a_ptr.a.value != 0 {
            b_ptr.a.value = (b_ptr.a.value / a_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, B) => {
          if a_ptr.b.value != 0 {
            b_ptr.b.value = (b_ptr.b.value / a_ptr.b.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, AB) => {
          if a_ptr.a.value != 0 {
            b_ptr.b.value = (b_ptr.b.value / a_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, BA) => {
          if a_ptr.b.value != 0 {
            b_ptr.a.value = (b_ptr.a.value / a_ptr.b.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Div, X) => {
          if a_ptr.a.value != 0 {
            b_ptr.b.value = (b_ptr.b.value / a_ptr.a.value) % size;
          }

          if a_ptr.b.value != 0 {
            b_ptr.a.value = (b_ptr.a.value / a_ptr.b.value) % size;
          }

          if a_ptr.a.value == 0 || a_ptr.b.value == 0 {
            None
          } else {
            Some(1)
          }
        }
        (Div, F) | (Div, I) => {
          if a_ptr.a.value != 0 {
            b_ptr.a.value = (b_ptr.a.value / a_ptr.a.value) % size;
          }

          if a_ptr.b.value != 0 {
            b_ptr.b.value = (b_ptr.b.value / a_ptr.b.value) % size;
          }

          if a_ptr.a.value == 0 || a_ptr.b.value == 0 {
            None
          } else {
            Some(1)
          }
        }

        // Modulo instructions
        (Mod, A) => {
          if a_ptr.a.value != 0 {
            b_ptr.a.value = (b_ptr.a.value % a_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Mod, B) => {
          if a_ptr.b.value != 0 {
            b_ptr.b.value = (b_ptr.b.value % a_ptr.b.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Mod, AB) => {
          if a_ptr.a.value != 0 {
            b_ptr.b.value = (b_ptr.b.value % a_ptr.a.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Mod, BA) => {
          if a_ptr.b.value != 0 {
            b_ptr.a.value = (b_ptr.a.value % a_ptr.b.value) % size;
            Some(1)
          } else {
            None
          }
        }
        (Mod, X) => {
          if a_ptr.a.value != 0 {
            b_ptr.b.value = (b_ptr.b.value % a_ptr.a.value) % size;
          }

          if a_ptr.b.value != 0 {
            b_ptr.a.value = (b_ptr.a.value % a_ptr.b.value) % size;
          }

          if a_ptr.a.value == 0 || a_ptr.b.value == 0 {
            None
          } else {
            Some(1)
          }
        }
        (Mod, F) | (Mod, I) => {
          if a_ptr.a.value != 0 {
            b_ptr.a.value = (b_ptr.a.value % a_ptr.a.value) % size;
          }

          if a_ptr.b.value != 0 {
            b_ptr.b.value = (b_ptr.b.value % a_ptr.b.value) % size;
          }

          if a_ptr.a.value == 0 || a_ptr.b.value == 0 {
            None
          } else {
            Some(1)
          }
        }

        // Jump instructions
        (Jmp, _) => Some(instr.a.value),

        (Jmz, A) | (Jmz, BA) => {
          if b_ptr.a.value == 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Jmz, B) | (Jmz, AB) => {
          if b_ptr.b.value == 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Jmz, F) | (Jmz, X) | (Jmz, I) => {
          if b_ptr.a.value == 0 && b_ptr.b.value == 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }

        // Always in B-mode
        (Jmn, A) | (Jmn, BA) => {
          if b_ptr.a.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Jmn, B) | (Jmn, AB) => {
          if b_ptr.b.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Jmn, F) | (Jmn, X) | (Jmn, I) => {
          if b_ptr.a.value != 0 && b_ptr.b.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }

        (Djn, A) | (Djn, BA) => {
          b_ptr.a.value = (b_ptr.a.value + size - 1) % size;
          // immutable increment a field of instr by one,
          let instr = Instruction {
            a: Field {
              value: (instr.a.value + size - 1) % size,
              ..instr.a
            },
            ..instr
          };

          if b_ptr.a.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Djn, B) | (Djn, AB) => {
          b_ptr.b.value = (b_ptr.b.value + size - 1) % size;
          // immutable increment a field of instr by one,
          let instr = Instruction {
            b: Field {
              value: (instr.b.value + size - 1) % size,
              ..instr.b
            },
            ..instr
          };

          if b_ptr.a.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }
        (Djn, F) | (Djn, X) | (Djn, I) => {
          b_ptr.b.value = (b_ptr.b.value + size - 1) % size;
          b_ptr.a.value = (b_ptr.a.value + size - 1) % size;
          // immutable increment a field of instr by one,
          let instr = Instruction {
            a: Field {
              value: (instr.a.value + size - 1) % size,
              ..instr.a
            },
            b: Field {
              value: (instr.b.value + size - 1) % size,
              ..instr.b
            },
            ..instr
          };

          if b_ptr.a.value != 0 || b_ptr.b.value != 0 {
            Some(instr.a.value)
          } else {
            Some(1)
          }
        }

        // Split instructions
        (Spl, _) => {
          // Start new thread by queuing new program counter
          threads.push_back(pc + instr.a.value);
          Some(1)
        }

        //
        (Seq, A) | (Cmp, A) => {
          if a_ptr.a.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, B) | (Cmp, B) => {
          if a_ptr.b.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, AB) | (Cmp, AB) => {
          if a_ptr.a.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, BA) | (Cmp, BA) => {
          if a_ptr.b.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, F) | (Cmp, F) => {
          if a_ptr.a.value == b_ptr.a.value && a_ptr.b.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, X) | (Cmp, X) => {
          if a_ptr.a.value == b_ptr.b.value && a_ptr.b.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Seq, I) | (Cmp, I) => {
          if a_ptr == *b_ptr {
            Some(2)
          } else {
            Some(1)
          }
        }

        (Slt, A) => {
          if a_ptr.a.value < b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Slt, B) => {
          if a_ptr.b.value < b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Slt, AB) => {
          if a_ptr.a.value < b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Slt, BA) => {
          if a_ptr.b.value < b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Slt, F) | (Slt, I) => {
          if a_ptr.a.value < b_ptr.a.value && a_ptr.b.value < b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Slt, X) => {
          if a_ptr.a.value < b_ptr.b.value && a_ptr.b.value < b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }

        (Sne, A) => {
          if a_ptr.a.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, B) => {
          if a_ptr.b.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, AB) => {
          if a_ptr.a.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, BA) => {
          if a_ptr.b.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, F) => {
          if a_ptr.a.value == b_ptr.a.value && a_ptr.b.value == b_ptr.b.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, X) => {
          if a_ptr.a.value == b_ptr.b.value && a_ptr.b.value == b_ptr.a.value {
            Some(2)
          } else {
            Some(1)
          }
        }
        (Sne, I) => {
          if a_ptr == *b_ptr {
            Some(2)
          } else {
            Some(1)
          }
        }

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
      threads.push_back(pc + offset % size);
    }

    // requeue the process if there are still threads
    if !threads.is_empty() {
      self.proceses.push_back((id, pspace, threads));
    }
  }

  /// Return all queues along with their process id
  pub fn processes(
    &self,
  ) -> impl Iterator<
    Item = (
      &Pid,
      impl Iterator<Item = &Instruction>,
      impl Iterator<Item = &Address>,
    ),
  > {
    self
      .proceses
      .iter()
      .map(|(id, p, q)| (id, p.iter(), q.iter())) // TODO: what to do with psace
  }

  pub fn reset(&mut self) -> &mut Self {
    // clear memory
    self
      .memory
      .iter_mut()
      .for_each(|x| *x = Instruction::default());

    // clear process queue
    self.proceses.clear();
    self
  }
}

// TODO: define default later
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarsBuilder {
  max_processes: usize,
  /// Size of private storage
  p_space_size: usize,
  size: usize,
}

impl MarsBuilder {
  /// Create a new `MarsBuilder` with required parameters
  pub fn new(size: usize) -> Self {
    MarsBuilder {
      size,
      ..Self::default()
    }
  }

  pub fn max_processes(self, value: usize) -> Self {
    Self {
      max_processes: value,
      ..self
    }
  }

  pub fn p_space_size(self, value: usize) -> Self {
    Self {
      p_space_size: value,
      ..self
    }
  }

  /// Build a `Mars`
  pub fn build(&self) -> Mars {
    let memory = vec![Instruction::default(); self.size];

    Mars {
      memory,
      p_space_size: self.p_space_size,
      proceses: VecDeque::new(),
    }
  }
}

impl Default for Mars {
  fn default() -> Self {
    Mars {
      memory: vec![Instruction::default(); 8000], // Make this a const
      p_space_size: 8,
      /// TODO: make this a const
      proceses: VecDeque::new(),
    }
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

impl AddressingMode {
  pub fn is_immediate(&self) -> bool {
    match *self {
      Immediate => true,
      _ => false,
    }
  }
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

#[cfg(test)]
mod test {
  use super::*;

  /// Imp: copies iteself sequentially
  const IMP: &[Instruction] = &[Instruction {
    op: OpField {
      mode: Opmode::I,
      code: Opcode::Mov,
    },
    a: Field {
      value: 0,
      mode: AddressingMode::Direct,
    },
    b: Field {
      value: 1,
      mode: AddressingMode::Direct,
    },
  }];

  #[test]
  fn processes_switching() {
    let mut mars = Mars::default();
    mars.load_program(IMP, 1);
    mars.load_program(&[Instruction::default()], 5); // will kill thread
    mars.load_program(IMP, 10);

    assert!(mars.processes().nth(0).unwrap().2.nth(0).unwrap() == &1);
    assert!(mars.processes().nth(1).unwrap().2.nth(0).unwrap() == &5);
    assert!(mars.processes().nth(2).unwrap().2.nth(0).unwrap() == &10);

    mars.step();

    // first inline should move to the back after being incremented
    assert!(mars.processes().nth(0).unwrap().2.nth(0).unwrap() == &5);
    assert!(mars.processes().nth(1).unwrap().2.nth(0).unwrap() == &10);
    assert!(mars.processes().nth(2).unwrap().2.nth(0).unwrap() == &2);

    mars.step();

    // first inline should move. The process that executed a dat should have
    // been killed
    assert!(mars.processes().nth(0).unwrap().2.nth(0).unwrap() == &10);
    assert!(mars.processes().nth(1).unwrap().2.nth(0).unwrap() == &2);
    assert!(mars.processes().nth(2).is_none());
  }

  #[test]
  fn loading_creates_new_queue() {
    let program = [Instruction::default(); 1000];
    let mut mars = Mars::default();

    let expected_queue = VecDeque::from(vec![1]);

    assert!(mars.process_count() == 0);
    mars.load_program(&program, 1);
    assert!(mars.process_count() == 1);
    {
      let (&id, _, first_queue) = mars.processes().nth(0).unwrap();
      assert_eq!(id, 0);
      assert_eq!(
        first_queue.cloned().collect::<VecDeque<usize>>(),
        expected_queue
      );
    }
    mars.load_program(&program, 1);
    assert!(mars.process_count() == 2);
    {
      let (&id, _, first_queue) = mars.processes().nth(1).unwrap();
      assert_eq!(id, 1);
      assert_eq!(
        first_queue.cloned().collect::<VecDeque<usize>>(),
        expected_queue
      );
    }
    mars.load_program(&program, 1);
    assert!(mars.process_count() == 3);
    {
      let (&id, _, first_queue) = mars.processes().nth(2).unwrap();
      assert_eq!(id, 2);
      assert_eq!(
        first_queue.cloned().collect::<VecDeque<usize>>(),
        expected_queue
      );
    }
  }

  #[test]
  fn loads_over_boundary() {
    let add = Instruction {
      op: OpField {
        code: Add,
        ..OpField::default()
      },
      ..Instruction::default()
    };
    let program = [add, add, add];
    let mut mars = Mars::default();
    mars.load_program(&program, 7999);
    assert!(mars.memory()[7998] == Instruction::default());
    assert!(mars.memory()[7999] == add);
    assert!(mars.memory()[0] == add);
    assert!(mars.memory()[1] == add);
    assert!(mars.memory()[2] == Instruction::default());
  }

  #[test]
  fn test_dat() {
    let mut mars = Mars::default();
    let program = [Instruction::default()];
    mars.load_program(&program, 0);
    mars.step();
    assert!(mars.process_count() == 0);
  }

  #[test]
  fn test_mov() {
    let mut mars = Mars::default();
    mars.load_program(&IMP, 0);
    mars.step();
    assert_eq!(mars.memory()[1], IMP[0]);
  }

  #[test]
  fn jmp() {
    let mut mars = Mars::default();
    let program = [Instruction {
      op: OpField {
        code: Jmp,
        mode: Opmode::B,
      },
      a: Field {
        mode: Direct,
        value: 8005,
      },
      b: Field {
        mode: Direct,
        value: 0,
      },
    }];
    mars.load_program(&program, 0);
    mars.step();
    assert_eq!(mars.pc(), Some(5));
  }
}
