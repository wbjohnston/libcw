//! Simulation runtime (aka `Core`) and tools to build a core

use std::collections::{VecDeque, HashMap};

use redcode::*;

pub type CoreResult<T> = Result<T, ()>;

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreEvent
{
    /// All processes terminated successfully
    Finished,

    /// Game ended in a tie
    Tied,

    /// Process split inner contains address of new pc
    Split,

    /// A process terminated
    Terminated(Pid),

    /// A process jumped address
    Jumped,

    /// Skipped happens in all `Skip if ...` instructions
    Skipped,

    /// Nothing happened
    Stepped,
}

/// Core wars runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core
{
    /// Core memory
    pub(super) memory:        Vec<Instruction>,

    /// Current process id being run
    pub(super) current_pid:   Pid,

    /// Current program counter
    pub(super) pc:            Address,

    /// Instruction register
    pub(super) ir:            Instruction,

    /// Current process queue
    pub(super) current_queue: VecDeque<Address>,

    /// Current numbered cycle core is executing
    pub(super) current_cycle: usize,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(Pid, VecDeque<Address>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<Pin, Vec<Instruction>>,

    /// Has the core finished executing
    pub(super) finished:      bool,

    // Runtime constraints
    /// Core version
    pub(super) version:       usize,

    /// Maximum of processes that can be on the process queue at any time
    pub(super) max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    pub(super) max_cycles:    usize,
}

impl Core
{
    /// Step forward one cycle
    pub fn step(&mut self) -> CoreResult<CoreEvent>
    {
        if self.finished() { // can't step after the core is halted
            return Err(());
        }

        if self.cycle() >= self.max_cycles() {
            self.finished = true;
            return Ok(CoreEvent::Tied)
        }

        // Fetch instruction
        self.ir = self.fetch(self.pc);
        let (a_mode, b_mode) = (self.ir.a.mode, self.ir.b.mode);

        // Preincrement phase
        // fetch direct target
        let direct_addr = self.calc_addr_offset(self.pc, self.ir.a.offset);
        let mut direct = self.fetch(direct_addr);

        match a_mode {
            AddressingMode::AIndirectPreDecrement => direct.a.offset -= 1,
            AddressingMode::BIndirectPreDecrement => direct.b.offset -= 1,
            _ => { /* Do nothing */ }
        };

        match b_mode {
            AddressingMode::AIndirectPreDecrement => direct.a.offset -= 1,
            AddressingMode::BIndirectPreDecrement => direct.b.offset -= 1,
            _ => { /* Do nothing */ }
        };

        // Execute instruction(updating the program counter and requeing it
        // are handled in this phase)
        let exec_event = self.execute();

        // PostIncrement phase
        // fetch direct target
        let direct_addr = self.calc_addr_offset(self.pc, self.ir.a.offset);
        let mut direct = self.fetch(direct_addr);

        match a_mode {
            AddressingMode::AIndirectPostIncrement => direct.a.offset += 1,
            AddressingMode::BIndirectPostIncrement => direct.b.offset += 1,
            _ => { /* Do nothing */ }
        };

        match b_mode {
            AddressingMode::AIndirectPostIncrement => direct.a.offset += 1,
            AddressingMode::BIndirectPostIncrement => direct.b.offset += 1,
            _ => { /* Do nothing */ }
        };

        // check if there are any more process queues running on the core
        if !self.current_queue.is_empty() {
            let q_entry = (self.pid(), self.current_queue.clone());
            self.process_queue.push_front(q_entry);
        }

        // check if there is only one PID remaining on the process queue
        if self.process_queue.len() <= 1 {
            self.finished = true;
            return Ok(CoreEvent::Finished);
        }

        // Fetch new queue
        let (pid, q)       = self.process_queue.pop_back().unwrap();
        self.current_queue = q;

        // Update pid and program counter
        self.pc          = self.current_queue.pop_back().unwrap();
        self.current_pid = pid;

        self.current_cycle += 1;
        Ok(exec_event)
    }

    /// Execute the instrcution in the `Instruction` register
    fn execute(&mut self) -> CoreEvent
    {
        let code = self.ir.op.code;

        match code {
            OpCode::Dat => self.exec_dat(),
            OpCode::Mov => self.exec_mov(),
            OpCode::Add => self.exec_add(),
            OpCode::Sub => self.exec_sub(),
            OpCode::Mul => self.exec_mul(),
            OpCode::Div => self.exec_div(),
            OpCode::Mod => self.exec_mod(),
            OpCode::Jmp => self.exec_jmp(),
            OpCode::Jmz => self.exec_jmz(),
            OpCode::Jmn => self.exec_jmn(),
            OpCode::Djn => self.exec_djn(),
            OpCode::Spl => self.exec_spl(),
            OpCode::Seq => self.exec_seq(),
            OpCode::Sne => self.exec_sne(),
            OpCode::Slt => self.exec_slt(),
            OpCode::Ldp => self.exec_ldp(),
            OpCode::Stp => self.exec_stp(),
            OpCode::Nop => self.exec_nop(),
        }
    }

    /// Has the core finished its execution. This can mean either a tie has
    /// occurred or a warrior has emerged victoriors
    pub fn finished(&mut self) -> bool
    {
        self.finished
    }

    /// Get `Pid` currently executing on the core
    pub fn pc(&self) -> Address
    {
        self.pc.clone()
    }

    /// Get the program counters for all processes
    pub fn pcs(&self) -> Vec<Address>
    {
        unimplemented!();
    }

    /// Current cycle core is executing
    pub fn cycle(&self) -> usize
    {
        self.current_cycle
    }

    /// Get the current `Pid` executing
    pub fn pid(&self) -> Pid
    {
        self.current_pid
    }

    /// Get all `Pid`s that are currently active
    pub fn pids(&self) -> Vec<Pid>
    {
        self.process_queue.iter().map(|&(pid, _)| pid).collect()
    }

    /// Size of memory
    pub fn size(&self) -> usize
    {
        self.memory.len()
    }

    /// Version of core multiplied by `100`
    pub fn version(&self) -> usize
    {
        self.version
    }

    /// Maximum number of processes that can be in the core queue
    pub fn max_processes(&self) -> usize
    {
        self.max_processes
    }

    /// Maximum number of cycles before a tie is declared
    pub fn max_cycles(&self) -> usize
    {
        self.max_cycles
    }

    /// Get immutable reference to memory
    pub fn memory(&self) -> &[Instruction]
    {
        &self.memory.as_slice()
    }

    /// Get the number of processes currently running
    pub fn process_count(&self) -> usize
    {
        // count length of all local process queues in the global pqueue
        self.process_queue.iter().fold(0, |acc, &(_, ref x)| acc + x.len())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Address resolution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Calculate the address after adding an offset
    ///
    /// # Arguments
    /// * `base`: base address
    /// * `offset`: distance from base to calculate
    #[inline]
    fn calc_addr_offset(&self, base: Address, offset: Offset) -> Address
    {
        if offset < 0 {
            (base.wrapping_sub(-offset as Address) % self.size() as Address)
        } else {
            (base.wrapping_add(offset as Address) % self.size() as Address)
        }
    }

    /// Get the effective of address of the current `Instruction`. This takes
    /// into account the addressing mode of the field used
    ///
    /// # Arguments
    /// * `use_a_field`: should the A field be used for calculation, or B
    #[inline]
    fn effective_addr(&self, use_a_field: bool) -> Address
    {
        use self::AddressingMode::*;

        // fetch the addressing mode and offset
        let (mode, offset) = {
            let field = if use_a_field { self.ir.a } else { self.ir.b };
            (field.mode, field.offset)
        };

        let direct = self.fetch(self.calc_addr_offset(self.pc, offset));

        match mode {
            Immediate => self.pc,
            Direct => self.calc_addr_offset(self.pc, offset),
            AIndirect
                | AIndirectPreDecrement
                | AIndirectPostIncrement =>
                self.calc_addr_offset(self.pc, direct.a.offset + offset),
            BIndirect
                | BIndirectPreDecrement
                | BIndirectPostIncrement =>
                self.calc_addr_offset(self.pc, direct.b.offset + offset),
        }
    }

    /// Get the effective of address of the current `Instruction`'s A Field
    ///
    /// An alias for `Core::effective_addr(true)`
    fn effective_addr_a(&self) -> Address
    {
        self.effective_addr(true)
    }

    /// Get the effective of address of the current `Instruction`'s A Field
    ///
    /// An alias for `Core::effective_addr(false)`
    fn effective_addr_b(&self) -> Address
    {
        self.effective_addr(false)
    }

    ////////////////////////////////////////////////////////////////////////////
    // Program counter utility functions
    ////////////////////////////////////////////////////////////////////////////

    /// Move the program counter forward
    fn step_pc(&mut self) -> CoreEvent
    {
        self.pc = (self.pc + 1) % self.size() as Address;
        CoreEvent::Stepped
    }

    /// Move the program counter forward twice
    fn skip_pc(&mut self) -> CoreEvent
    {
        self.pc = (self.pc + 2) % self.size() as Address;
        CoreEvent::Skipped
    }

    /// Jump the program counter by an offset
    ///
    /// # Arguments
    /// * `offset`: amount to jump
    fn jump_pc(&mut self, offset: Offset) -> CoreEvent
    {
        self.pc = self.calc_addr_offset(self.pc, offset);
        CoreEvent::Jumped
    }

    /// Move the program counter forward by one and then queue the program
    /// counter onto the current queue
    fn step_and_queue_pc(&mut self) -> CoreEvent
    {
        self.step_pc();
        self.current_queue.push_front(self.pc);
        CoreEvent::Stepped
    }

    /// Move the program counter forward twice and then queue the program
    /// counter onto the current queue
    fn skip_and_queue_pc(&mut self) -> CoreEvent
    {
        self.skip_pc();
        self.current_queue.push_front(self.pc);
        CoreEvent::Skipped
    }

    /// Jump the program counter by an offset and then queue the program
    /// count onto the current queue
    ///
    /// # Arguments
    /// * `offset`: amount to jump by
    fn jump_and_queue_pc(&mut self, offset: Offset) -> CoreEvent
    {
        self.jump_pc(offset);
        self.current_queue.push_front(self.pc);
        CoreEvent::Jumped
    }

    ////////////////////////////////////////////////////////////////////////////
    // Storage and retrieval functions
    ////////////////////////////////////////////////////////////////////////////

    /// Store an `Instruction` in memory
    ///
    /// # Arguments
    /// * `addr`: address to store
    /// * `instr`: instruction to store
    fn store(&mut self, addr: Address, instr: Instruction)
    {
        let mem_size = self.size();
        self.memory[addr as usize % mem_size] = instr;
    }

    /// Store an `Instruction` into the memory location pointed at by the A
    /// field of the instruction loaded into the instruction register
    ///
    /// # Arguments
    /// * `instr`: `Instruction` to store
    fn store_effective_a(&mut self, instr: Instruction)
    {
        let eff_addr = self.effective_addr_a();
        self.store(eff_addr, instr)
    }

    /// Store an `Instruction` into the memory location pointed at by the B
    /// field of the instruction loaded into the instruction register
    ///
    /// # Arguments
    /// * `instr`: `Instruction` to store
    fn store_effective_b(&mut self, instr: Instruction)
    {
        let eff_addr = self.effective_addr_b();
        self.store(eff_addr, instr)
    }

    /// Fetch copy of instruction in memory
    ///
    /// # Arguments
    /// * `addr`: adress to fetch
    fn fetch(&self, addr: Address) -> Instruction
    {
        self.memory[addr as usize % self.size()]
    }

    /// Fetch copy of instruction pointed at by the A field of the instruction
    /// loaded into the instruction register
    fn fetch_effective_a(&self) -> Instruction
    {
        self.fetch(self.effective_addr_a())
    }

    /// Fetch copy of instruction pointed at by the B field of the instruction
    /// loaded into the instruction register
    fn fetch_effective_b(&self) -> Instruction
    {
        self.fetch(self.effective_addr_b())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Instruction execution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Execute `dat` instruction
    ///
    /// Supported OpModes: None
    fn exec_dat(&self) -> CoreEvent
    {
        CoreEvent::Terminated(self.pid())
    }

    /// Execute `mov` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a = a.a,
            OpMode::B => b.b = a.b,
            OpMode::AB =>b.a = a.b,
            OpMode::BA =>b.b = a.a,
            OpMode::F =>
            {
                b.a = a.a;
                b.b = a.b;
            },
            OpMode::X =>
            {
                b.a = a.b;
                b.b = a.a;
            },
            OpMode::I => b = a
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `add` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a.offset += a.a.offset,
            OpMode::B => b.b.offset += a.b.offset,
            OpMode::BA =>b.a.offset += a.b.offset,
            OpMode::AB =>b.b.offset += a.a.offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset += a.a.offset;
                b.b.offset += a.b.offset;
            },
            OpMode::X =>
            {
                b.b.offset += a.a.offset;
                b.a.offset += a.b.offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a.offset -= a.a.offset,
            OpMode::B => b.b.offset -= a.b.offset,
            OpMode::BA =>b.a.offset -= a.b.offset,
            OpMode::AB =>b.b.offset -= a.a.offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset -= a.a.offset;
                b.b.offset -= a.b.offset;
            },
            OpMode::X =>
            {
                b.b.offset -= a.a.offset;
                b.a.offset -= a.b.offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `mul` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a.offset *= a.a.offset,
            OpMode::B => b.b.offset *= a.b.offset,
            OpMode::BA =>b.a.offset *= a.b.offset,
            OpMode::AB =>b.b.offset *= a.a.offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset *= a.a.offset;
                b.b.offset *= a.b.offset;
            },
            OpMode::X =>
            {
                b.b.offset *= a.a.offset;
                b.a.offset *= a.b.offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `div` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a.offset /= a.a.offset,
            OpMode::B => b.b.offset /= a.b.offset,
            OpMode::BA =>b.a.offset /= a.b.offset,
            OpMode::AB =>b.b.offset /= a.a.offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset /= a.a.offset;
                b.b.offset /= a.b.offset;
            },
            OpMode::X =>
            {
                b.b.offset /= a.a.offset;
                b.a.offset /= a.b.offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `mod` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(&mut self) -> CoreEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A => b.a.offset %= a.a.offset,
            OpMode::B => b.b.offset %= a.b.offset,
            OpMode::BA =>b.a.offset %= a.b.offset,
            OpMode::AB =>b.b.offset %= a.a.offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset %= a.a.offset;
                b.b.offset %= a.b.offset;
            },
            OpMode::X =>
            {
                b.b.offset %= a.a.offset;
                b.a.offset %= a.b.offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmp(&mut self) -> CoreEvent
    {
        match self.ir.a.mode {
            AddressingMode::Immediate
                | AddressingMode::Direct =>
            {
                let offset = self.ir.a.offset;
                self.jump_and_queue_pc(offset);
            }
            // TODO
            _ => unimplemented!()
        };

        CoreEvent::Jumped
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(&mut self) -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(&mut self) -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(&mut self) -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(&mut self) -> CoreEvent
    {
        if self.process_count() < self.max_processes(){
            let target = self.effective_addr_a();
            self.current_queue.push_front(target);

            self.step_and_queue_pc();
            CoreEvent::Split
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&mut self) -> CoreEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.op.mode {
            OpMode::A       => a.a.offset == b.b.offset,
            OpMode::B       => a.b.offset == b.b.offset,
            OpMode::BA      => a.a.offset == b.b.offset,
            OpMode::AB      => a.b.offset == b.a.offset,
            OpMode::X       => a.b.offset == b.a.offset &&
                               a.a.offset == b.b.offset,
            OpMode::F
                | OpMode::I => a.a.offset == b.a.offset &&
                               a.b.offset == b.b.offset,
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(&mut self) -> CoreEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.op.mode {
            OpMode::A       => a.a.offset != b.b.offset,
            OpMode::B       => a.b.offset != b.b.offset,
            OpMode::BA      => a.a.offset != b.b.offset,
            OpMode::AB      => a.b.offset != b.a.offset,
            OpMode::X       => a.b.offset != b.a.offset &&
                               a.a.offset != b.b.offset,
            OpMode::F
                | OpMode::I => a.a.offset != b.a.offset &&
                               a.b.offset != b.b.offset,
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(&mut self) -> CoreEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.op.mode {
            OpMode::A       => a.a.offset < b.b.offset,
            OpMode::B       => a.b.offset < b.b.offset,
            OpMode::BA      => a.a.offset < b.b.offset,
            OpMode::AB      => a.b.offset < b.a.offset,
            OpMode::X       => a.b.offset < b.a.offset &&
                               a.a.offset < b.b.offset,
            OpMode::F
                | OpMode::I => a.a.offset < b.a.offset &&
                               a.b.offset < b.b.offset,
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(&mut self) -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(&mut self) -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute 'nop' instruction
    fn exec_nop(&mut self) -> CoreEvent
    {
        self.step_and_queue_pc()
    }
}

