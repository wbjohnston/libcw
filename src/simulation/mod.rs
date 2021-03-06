//! Your one-stop shop for everything Core Wars
use {
  itertools::assert_equal,
  redcode::{
    self, Address, AddressingMode, AddressingMode::*, Field, IncrementMode, Instruction, OpCode,
    OpCode::*, OpField, OpMode, OpMode::*,
  },
  std::collections::VecDeque,
  std::rc::Rc,
};

const MARS_DEFAULT_SIZE: usize = 8000;
const MARS_DEFAULT_P_SPACE_SIZE: usize = 8;

/// A process id
pub type Pid = usize;

/// A collection on queued threads
pub type Threads = VecDeque<Address>;

/// Process storage
pub type PSpace = Rc<Vec<Address>>;

/// A mars process
pub type Process = (Pid, PSpace, Threads);

/// A corewars simulator
#[derive(Debug, Clone)]
pub struct Mars {
  memory: Vec<Instruction>,
  p_space_size: usize,
  cycle: usize,
  processes: VecDeque<Process>,
}

impl Mars {
  /// Return the next program counter
  pub fn pc(&self) -> Option<Address> {
    self
      .processes
      .front()
      .and_then(|(_, _, process)| process.front().cloned())
  }

  /// Return size of mars' memory
  pub fn size(&self) -> usize {
    self.memory.len()
  }

  /// Return the currect cpu cycle
  pub fn cycle(&self) -> usize {
    self.cycle
  }

  /// Return the next process id that will execute
  pub fn pid(&self) -> Option<Pid> {
    self.processes.front().map(|&(pid, _, _)| pid)
  }

  /// Return all active procces ids
  pub fn pids(&self) -> impl Iterator<Item = Pid> + '_ {
    self.processes.iter().cloned().map(|(pid, _, _)| pid)
  }

  /// Returns each processes resources zipped with its pid
  pub fn processes(&self) -> impl Iterator<Item = &Process> {
    self.processes.iter()
  }

  /// Return process queues zipped with the owning process' id
  pub fn process_queues(&self) -> impl Iterator<Item = (Pid, impl Iterator<Item = &Address>)> {
    self
      .processes
      .iter()
      .map(|(pid, _, queue)| (*pid, queue.iter()))
  }

  /// Return process private storage(pspace) zipped with the owning process' id
  pub fn process_pspaces(&self) -> impl Iterator<Item = (usize, &[Address])> {
    self
      .processes
      .iter()
      .map(|(pid, pspace, _)| (*pid, pspace.as_slice()))
  }

  /// Returns the current number of processes
  pub fn process_count(&self) -> usize {
    self.processes.len()
  }

  /// Return the number of threads each process has along with the process id
  pub fn thread_count(&self) -> impl Iterator<Item = (Pid, usize)> + '_ {
    self
      .processes
      .iter()
      .map(|&(id, _, ref threads)| (id, threads.len()))
  }

  /// Return a view of the Mars' memory
  pub fn memory(&self) -> &[Instruction] {
    self.memory.as_slice()
  }

  pub fn set_memory(&mut self, instructions: &[Instruction], address: Address) {
    let size = self.size();
    for i in 0..instructions.len() {
      self.memory[((address as usize + i) % size)] = instructions[i];
    }
  }

  pub fn load_program(&mut self, program: &[Instruction], address: Address) -> Pid {
    let pspace = Rc::new(vec![]);
    self.load_program_with_pspace(program, address, pspace)
  }

  /// Load multiple programs in different locations with the same pspace
  ///
  /// # Returns
  /// A slice containing all of the created process ids, in order
  pub fn load_programs_with_shared_pspace(
    &mut self,
    programs: &[&[Instruction]],
    addresses: &[Address],
  ) -> Vec<Pid> {
    let pspace = Rc::new(vec![]);
    let mut pids = vec![];
    for (program, &addr) in programs.iter().zip(addresses.iter()) {
      let pid = self.load_program_with_pspace(program, addr, pspace.clone());
      pids.push(pid);
    }

    pids
  }

  fn load_program_with_pspace(
    &mut self,
    program: &[Instruction],
    address: Address,
    pspace: PSpace,
  ) -> Pid {
    let pid = self.processes.len();
    let mut threads = VecDeque::new();
    self.set_memory(program, address);
    threads.push_back(address);
    self.processes.push_back((pid, pspace, threads));
    pid
  }

  /// Step forward one clock cycle
  ///
  /// # Panics
  /// panics if there are no processes in the Mars
  ///
  /// # Returns
  /// `Some(pid)` if a process with id `pid` was killed. Otherwise `None`
  pub fn step(&mut self) -> Option<Pid> {
    assert!(
      !self.processes.is_empty(),
      "cannot execute with empty process queue"
    );
    self.cycle += 1; // increment cycle
    let size = self.memory.len() as Address;
    let (pid, mut pspace, mut threads) = self // dequeue the next process
      .processes
      .pop_front()
      .expect("cannot step if no processes exist");
    let pc = threads // dequeue the next thread's program counter
      .pop_front()
      .expect("cannot execute a process with no threads");
    let instr = self.memory[(pc % size) as usize]; // fetch instruction from memory

    let a_target_address = self.resolve_address(pc, instr.a.value, size, instr.a.mode);
    let b_target_address = self.resolve_address(pc, instr.b.value, size, instr.b.mode);

    // Preincrement phase
    match instr.a.mode {
      AIndirect(IncrementMode::PreDecrement) => {
        self.memory[((pc + instr.a.value) % size) as usize].a.value -= 1
      }
      BIndirect(IncrementMode::PreDecrement) => {
        self.memory[((pc + instr.a.value) % size) as usize].b.value -= 1
      }
      _ => {}
    }

    match instr.b.mode {
      AIndirect(IncrementMode::PreDecrement) => {
        self.memory[((pc + instr.b.value) % size) as usize].a.value -= 1
      }
      BIndirect(IncrementMode::PreDecrement) => {
        self.memory[((pc + instr.b.value) % size) as usize].b.value -= 1
      }
      _ => {}
    }

    // Execution phase, if a valid instruction was executed Some(offset) is
    // is returned. the offset is the amount that the program counter is
    // incremented
    let maybe_offset = {
      let a_ptr = self.memory[(a_target_address % size) as usize];
      let b_ptr = &mut self.memory[(b_target_address % size) as usize];

      // Instruction execution phase
      use OpMode::*;
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
      AIndirect(IncrementMode::PostIncrement) => {
        self.memory[(pc + instr.a.value) as usize].a.value += 1
      }
      BIndirect(IncrementMode::PostIncrement) => {
        self.memory[(pc + instr.a.value) as usize].b.value += 1
      }
      _ => {}
    }

    match instr.b.mode {
      AIndirect(IncrementMode::PostIncrement) => {
        self.memory[(pc + instr.b.value) as usize].a.value += 1
      }
      BIndirect(IncrementMode::PostIncrement) => {
        self.memory[(pc + instr.b.value) as usize].b.value += 1
      }
      _ => {}
    }

    // requeue the program counter if the thread wasn't killed
    if let Some(offset) = maybe_offset {
      threads.push_back((pc + offset) % size);
    }

    // requeue the process if there are still threads
    if !threads.is_empty() {
      self.processes.push_back((pid, pspace, threads));
      None
    } else {
      Some(pid)
    }
  }

  /// Reset the mars
  ///
  /// Clears the processe queue and memory
  pub fn reset(&mut self) -> &mut Self {
    // clear memory
    self
      .memory
      .iter_mut()
      .for_each(|x| *x = Instruction::default());

    // clear process queue
    self.processes.clear();
    self
  }

  /// Return an address calculated relative to the given program counter and a
  /// given mode
  ///
  /// # Params
  /// * `pc`: program counter
  /// * `offset`: offset to add to program counter
  /// * `size`: size of core
  /// * `addr_mode`: method to resolve address
  fn resolve_address(
    &self,
    pc: Address,
    offset: Address,
    size: u32,
    addr_mode: AddressingMode,
  ) -> Address {
    pc + match addr_mode {
      Direct => offset,
      AIndirect(..) => offset + self.memory[((pc + offset) % size) as usize].a.value,
      BIndirect(..) => offset + self.memory[((pc + offset) % size) as usize].b.value,
      Immediate => 0,
    }
  }

  fn normalize(&self, instruction: Instruction) -> Instruction {
    Instruction {
      a: Field {
        value: instruction.a.value % self.memory.len() as Address,
        ..instruction.a
      },
      b: Field {
        value: instruction.b.value % self.memory.len() as Address,
        ..instruction.b
      },
      ..instruction
    }
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
      processes: VecDeque::new(),
      ..Mars::default()
    }
  }
}

impl Default for Mars {
  fn default() -> Self {
    Mars {
      memory: vec![Instruction::default(); MARS_DEFAULT_SIZE], // Make this a const
      p_space_size: MARS_DEFAULT_P_SPACE_SIZE,
      cycle: 0,
      /// TODO: make this a const
      processes: VecDeque::new(),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  fn you_know_what_it_is(program: &[Instruction], addr: Address) -> Mars {
    let mut mars = Mars::default();
    mars.load_program(program, addr);
    mars.step();
    mars
  }

  /// Imp: copies iteself sequentially
  const IMP: &[Instruction] = &[Instruction {
    op: OpField {
      mode: OpMode::I,
      code: OpCode::Mov,
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
  fn mov_a() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, A, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      a: program[1].a,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_b() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, B, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      b: program[1].b,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_ab() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, AB, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      b: program[1].a,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_ba() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, BA, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      a: program[1].b,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_f() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, F, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      a: program[1].a,
      b: program[1].b,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_i() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, I, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = program[1];
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  #[test]
  fn mov_x() {
    let mut mars = Mars::default();
    let target_addr = mars.size() + 50;
    let expected_addr = target_addr % mars.size();
    let program = &[
      Instruction::new(Mov, X, Direct, 1, Direct, target_addr as Address),
      Instruction::new(Nop, F, Immediate, 100, Immediate, 100),
    ];
    mars.load_program(program, 0);
    mars.step();

    let expected = Instruction {
      a: program[1].b,
      b: program[1].a,
      ..Instruction::default()
    };
    assert_eq!(expected, mars.memory()[expected_addr])
  }

  // TODO: implement tests for other instructions

  #[test]
  fn processes_switching() {
    let mut mars = Mars::default();
    mars.load_program(IMP, 1);
    mars.load_program(&[Instruction::default()], 5); // will kill thread
    mars.load_program(IMP, 10);

    assert!(mars.processes().nth(0).unwrap().2.iter().nth(0).unwrap() == &1);
    assert!(mars.processes().nth(1).unwrap().2.iter().nth(0).unwrap() == &5);
    assert!(mars.processes().nth(2).unwrap().2.iter().nth(0).unwrap() == &10);

    mars.step();

    // first inline should move to the back after being incremented
    assert!(mars.processes().nth(0).unwrap().2.iter().nth(0).unwrap() == &5);
    assert!(mars.processes().nth(1).unwrap().2.iter().nth(0).unwrap() == &10);
    assert!(mars.processes().nth(2).unwrap().2.iter().nth(0).unwrap() == &2);

    mars.step();

    // first inline should move. The process that executed a dat should have
    // been killed
    assert!(mars.processes().nth(0).unwrap().2.iter().nth(0).unwrap() == &10);
    assert!(mars.processes().nth(1).unwrap().2.iter().nth(0).unwrap() == &2);
    assert!(mars.processes().nth(2).is_none());
  }

  #[ignore]
  #[test]
  fn loading_creates_new_queue() {
    let addr1 = 0;
    let addr2 = 1000;
    let addr3 = 2000;
    let addr4 = 3000;

    let program = [Instruction::default(); 1000];
    let mut mars = Mars::default();
    let pid1 = mars.load_program(&program, addr1);
    let pid2 = mars.load_program(&program, addr2);
    let pid3 = mars.load_program(&program, addr3);
    let pid4 = mars.load_program(&program, addr4);

    let expected_queue = {
      let mut q = VecDeque::new();
      q.push_back((pid1, Vec::<Address>::new(), VecDeque::from(vec![addr1])));
      q.push_back((pid2, Vec::<Address>::new(), VecDeque::from(vec![addr2])));
      q.push_back((pid3, Vec::<Address>::new(), VecDeque::from(vec![addr3])));
      q.push_back((pid4, Vec::<Address>::new(), VecDeque::from(vec![addr4])));

      q
    };
    // TODO: implement me
  }

  #[test]
  fn loads_over_boundary() {
    let add = Instruction::new(Add, I, Direct, 100, Direct, 100);
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
    let program = [Instruction::new(Dat, I, Direct, 0, Direct, 0)];
    let mars = you_know_what_it_is(&program, 0);
    assert!(mars.process_count() == 0);
  }

  #[test]
  fn test_mov_i() {
    let program = &[Instruction::new(Mov, I, Immediate, 0, Direct, 1)];
    let mars = you_know_what_it_is(program, 0);
    assert_eq!(mars.memory()[1], program[0]);
  }

  #[test]
  fn test_mov_f() {
    let program = &[Instruction::new(Mov, I, Immediate, 0, Direct, 1)];
    let mars = you_know_what_it_is(program, 0);
    assert_eq!(mars.memory()[1], program[0]);
  }

  #[test]
  fn test_add() {
    let program = [Instruction::new(Add, F, Immediate, 2, Direct, 1)];
    let mars = you_know_what_it_is(&program, 0);
    assert_eq!(
      mars.memory()[1],
      Instruction::new(Dat, I, Direct, 2, Direct, 1)
    )
  }

  #[test]
  fn test_jmp() {
    let program = [Instruction::new(Jmp, B, Direct, 8005, Direct, 0)];
    let mars = you_know_what_it_is(&program, 0);
    assert_eq!(mars.pc(), Some(5));
  }
}
