
use std::collections::{VecDeque, HashMap};

use redcode::*;

pub type SimulationResult<T> = Result<T, SimulationError>;
pub type LoadResult<T> = Result<T, LoadError>;

/// Errors that can occur during simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulationError
{
    /// Core was already halted
    Halted,
}

/// Errors that can occur during loading
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LoadError
{
    /// Validation error: program has invalid length
    InvalidLength,

    /// Validation error: invalid distance between programs
    InvalidDistance,

    /// Load cannot be called with no programs
    EmptyLoad
}

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulationEvent
{
    /// Game ended in a tie
    Tied,

    /// Process split inner contains address of new pc
    Split,

    /// A process terminated
    Terminated,

    /// A process jumped address
    Jumped,

    /// Skipped happens in all `Skip if ...` instructions
    Skipped,

    /// Nothing happened
    Stepped,
}

/// Mars wars runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mars
{
    /// Mars memory
    pub(super) memory:        Vec<Instruction>,

    /// Instruction register
    pub(super) ir:            Instruction,

    /// Current numbered cycle core is executing
    pub(super) cycle:         usize,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(Pid, VecDeque<Address>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<Pin, Vec<Offset>>,

    /// Has the core finished executing
    pub(super) halted:        bool,

    // Load constraints
    /// Maximum length of programs when loading
    pub(super) max_length:    usize,

    /// Minimum distance between programs when batch loading
    pub(super) min_distance:  usize,

    // Mars information (const)
    /// Mars version
    pub(super) version:       usize,

    /// Size of P-space
    pub(super) pspace_size:   usize,

    // Runtime constraints
    /// Maximum of processes that can be on the process queue at any time
    pub(super) max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    pub(super) max_cycles:    usize,
}

impl Mars
{
    /// Step forward one cycle
    pub fn step(&mut self) -> SimulationResult<SimulationEvent>
    {
        if self.halted() { // can't step after the core is halted
            return Err(SimulationError::Halted);
        }

        if self.cycle() >= self.max_cycles() {
            self.halted = true;
            return Ok(SimulationEvent::Tied)
        }

        let pc = self.pc().unwrap();

        // Fetch instruction
        self.ir = self.fetch(pc);
        let (a_mode, b_mode) = (self.ir.a.mode, self.ir.b.mode);

        // PostIncrement phase
        let predecrement = a_mode == AddressingMode::AIndirectPreDecrement ||
            a_mode == AddressingMode::BIndirectPreDecrement ||
            b_mode == AddressingMode::AIndirectPreDecrement ||
            b_mode == AddressingMode::BIndirectPreDecrement;

        // Preincrement phase
        if predecrement {
            // fetch direct target
            let a_addr = self.calc_addr_offset(pc, self.ir.a.offset);
            let b_addr = self.calc_addr_offset(pc, self.ir.b.offset);
            let mut a = self.fetch(a_addr);
            let mut b = self.fetch(b_addr);

            // FIXME: combine these into a single match statement
            match a_mode {
                AddressingMode::AIndirectPreDecrement => a.a.offset -= 1,
                AddressingMode::BIndirectPreDecrement => a.b.offset -= 1,
                _ => { /* Do nothing */ }
            };

            match b_mode {
                AddressingMode::AIndirectPreDecrement => b.a.offset -= 1,
                AddressingMode::BIndirectPreDecrement => b.b.offset -= 1,
                _ => { /* Do nothing */ }
            };
            self.store(a_addr, a);
            self.store(b_addr, b);
        }

        // Execute instruction(updating the program counter and requeing it
        // are handled in this phase)
        let exec_event = self.execute();

        // PostIncrement phase
        let postincrement = a_mode == AddressingMode::AIndirectPostIncrement ||
            a_mode == AddressingMode::BIndirectPostIncrement ||
            b_mode == AddressingMode::AIndirectPostIncrement ||
            b_mode == AddressingMode::BIndirectPostIncrement;

        if postincrement {
            // fetch direct target
            let a_addr = self.calc_addr_offset(pc, self.ir.a.offset);
            let b_addr = self.calc_addr_offset(pc, self.ir.b.offset);
            let mut a = self.fetch(a_addr);
            let mut b = self.fetch(b_addr);

            // FIXME: combine these into a single match statement
            match a_mode {
                AddressingMode::AIndirectPostIncrement => a.a.offset += 1,
                AddressingMode::BIndirectPostIncrement => a.b.offset += 1,
                _ => { /* Do nothing */ }
            };

            match b_mode {
                AddressingMode::AIndirectPostIncrement => b.a.offset += 1,
                AddressingMode::BIndirectPostIncrement => b.b.offset += 1,
                _ => { /* Do nothing */ }
            };
            // store result
            self.store(a_addr, a);
            self.store(b_addr, b);
        }

        // check if there are any more process queues running on the core
        if !self.current_queue().unwrap().is_empty() {
            let q = self.process_queue.pop_front().unwrap();
            self.process_queue.push_back(q);
        }

        // If no there are no processes left
        if self.process_queue.is_empty() {
            self.halted = true;
        } else {
            // Fetch new queue
            let q = self.process_queue.pop_front().unwrap();
            self.process_queue.push_back(q);
        }

        self.cycle += 1;
        Ok(exec_event)
    }

    /// Has the core finished its execution. This can mean either a tie has
    /// occurred or a warrior has emerged victoriors
    pub fn halted(&self) -> bool
    {
        self.halted
    }

    /// Halt the Mars
    pub fn halt(&mut self) -> &mut Self
    {
        self.halted = true;
        self
    }

    /// Reset the Mars's memory and the process queue
    pub fn reset(&mut self)
    {
        // reset memory
        for e in self.memory.iter_mut() {
            *e = Instruction::default();
        }

        self.process_queue.clear();

        self.cycle         = 0;
        self.ir            = Instruction::default();
        self.halted        = true;
    }

    /// Reset the Mar's memory, process queue, AND P-space
    pub fn reset_hard(&mut self)
    {
        self.pspace.clear();
        self.reset();
    }

    /// Load a program, checking only its length for validity
    /// # Arguments
    /// * `dest`: memory address program will be loaded to
    /// * `pin`: the `Pin` that the program will use to access it's private st
    ///     storage
    /// * `prog`: reference to the program to load
    ///
    /// # Return
    /// If the program is loaded successfully `Ok(()). Otherwise, the program 
    ///     was too long. This is the only load constraint that is checked in
    ///     a singleton load
    pub fn load(&mut self, dest: Address, pin: Option<Pin>, prog: &Program)
        -> LoadResult<()>
    {
        let valid_length = prog.len() <= self.max_length();
        if valid_length {
            // load program into memory
            let size = self.size();
            let pin = pin.unwrap_or(self.process_count() as Pid);
            for i in 0..prog.len() {
                self.memory[(i + dest as usize) % size] = prog[i];
            }

            // Create pspace
            self.pspace.insert(pin, vec![0; self.pspace_size]);

            // Add to process queue
            let mut q = VecDeque::new();
            q.push_front(dest);
            self.process_queue.push_front((pin, q));

            self.halted = false;
            Ok(())
        } else {
            Err(LoadError::InvalidLength)
        }
    }

    /// Load mutliple programs into the Mars, checking their spacing and their
    /// length
    /// # Arguments
    /// * `programs`: programs and load information loaded in a tuple, cannot
    ///     be empty
    /// # Return
    /// `Ok(())` if the load was successful, otherwise an error with the 
    ///     corresponding `SimulationError`
    pub fn load_batch(&mut self, programs: Vec<(Address, Option<Pin>, &Program)>)
        -> LoadResult<()>
    {
        if programs.is_empty() {
            return Err(LoadError::EmptyLoad);
        }

        let valid_margin = true; // TODO: actually validate distance

        if valid_margin {
            for &(dest, maybe_pin, ref prog) in programs.iter() {
                self.load(dest, maybe_pin, prog)?; 
            }

            Ok(())
        } else {
            Err(LoadError::InvalidDistance)
        }
    }

    /// Get `Pid` currently executing on the core
    pub fn pc(&self) -> Option<Address>
    {
        if let Some(q) = self.current_queue() {
            if let Some(pc) = q.front() {
                Some(pc.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the program counters for all processes
    pub fn pcs(&self) -> Vec<Address>
    {
        let mut pcs = vec![];

        for &(_, ref q) in &self.process_queue {
            pcs.extend(q.iter().cloned());
        }

        pcs
    }

    /// Current cycle core is executing
    pub fn cycle(&self) -> usize
    {
        self.cycle
    }

    /// Get the current `Pid` executing
    pub fn pid(&self) -> Option<Pid>
    {
        if let Some(&(pid, _)) = self.process_queue.front() {
            Some(pid)
        } else {
            None
        }
    }

    /// Get all `Pid`s that are currently active in the order they will be 
    /// executing
    pub fn pids(&self) -> Vec<Pid>
    {
        let mut pids = vec![];
        if let Some(pid) = self.pid() {
            pids.push(pid);
            pids.extend(self.process_queue.iter().map(|&(pid, _)| pid));
        } 
        pids
    }

    /// Size of memory
    pub fn size(&self) -> usize
    {
        self.memory.len()
    }

    /// Size of private storage space
    pub fn pspace_size(&self) -> usize
    {
        self.pspace_size
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

    /// Maximum number of instructions allowed in a program
    pub fn max_length(&self) -> usize
    {
        self.max_length
    }

    /// Minimum distance allowed between programs
    pub fn min_distance(&self) -> usize
    {
        self.min_distance
    }

    /// Get immutable reference to memory
    pub fn memory(&self) -> &[Instruction]
    {
        self.memory.as_slice()
    }

    /// Get the number of processes currently running
    pub fn process_count(&self) -> usize
    {
        self.process_queue.iter().map(|&(_, ref q)| q.len()).sum()
    }

    /// Fetch reference to current queue
    fn current_queue(&self) -> Option<&VecDeque<Address>>
    {
        if let Some(&(_, ref q)) = self.process_queue.front() {
            Some(q)
        } else {
            None
        }
    }

    /// Fetch mutable reference to current queue
    fn current_queue_mut(&mut self) -> Option<&mut VecDeque<Address>>
    {
        if let Some(&mut (_, ref mut q)) = self.process_queue.front_mut() {
            Some(q)
        } else {
            None
        }
    }

    /// Execute the instrcution in the `Instruction` register
    fn execute(&mut self) -> SimulationEvent
    {
        let e = match self.ir.op.code {
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
        };

        // pop old program counter
        self.current_queue_mut().unwrap().pop_front();
        e
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

        let pc = self.pc().unwrap();

        let direct = self.fetch(self.calc_addr_offset(pc, offset));

        match mode {
            Immediate => pc,
            Direct => self.calc_addr_offset(pc, offset),
            AIndirect
                | AIndirectPreDecrement
                | AIndirectPostIncrement =>
                self.calc_addr_offset(pc, direct.a.offset + offset),
            BIndirect
                | BIndirectPreDecrement
                | BIndirectPostIncrement =>
                self.calc_addr_offset(pc, direct.b.offset + offset),
        }
    }

    /// Get the effective of address of the current `Instruction`'s A Field
    ///
    /// An alias for `Mars::effective_addr(true)`
    fn effective_addr_a(&self) -> Address
    {
        self.effective_addr(true)
    }

    /// Get the effective of address of the current `Instruction`'s A Field
    ///
    /// An alias for `Mars::effective_addr(false)`
    fn effective_addr_b(&self) -> Address
    {
        self.effective_addr(false)
    }

    ////////////////////////////////////////////////////////////////////////////
    // Program counter utility functions
    ////////////////////////////////////////////////////////////////////////////

    /// Move the program counter forward
    fn step_pc(&mut self) -> SimulationEvent
    {
        let pc = self.pc().unwrap();
        *self.current_queue_mut().unwrap().front_mut().unwrap() =
            (pc + 1) % self.size() as Address;
        SimulationEvent::Stepped
    }

    /// Move the program counter forward twice
    fn skip_pc(&mut self) -> SimulationEvent
    {
        let pc =self.pc().unwrap();
        // TODO: Holy shit this is uuugggglllllyyyy
        *self.current_queue_mut().unwrap().front_mut().unwrap() = 
            (pc + 2) % self.size() as Address;
        SimulationEvent::Skipped
    }

    /// Jump the program counter by an offset
    ///
    /// # Arguments
    /// * `offset`: amount to jump
    fn jump_pc(&mut self, offset: Offset) -> SimulationEvent
    {
        let pc = self.pc().unwrap();
        // TODO: Holy shit this is uuugggglllllyyyy
        *self.current_queue_mut().unwrap().front_mut().unwrap() = 
            self.calc_addr_offset(pc, offset);
        SimulationEvent::Jumped
    }

    /// Move the program counter forward by one and then queue the program
    /// counter onto the current queue
    fn step_and_queue_pc(&mut self) -> SimulationEvent
    {
        self.step_pc();

        let pc = self.pc().unwrap();
        self.current_queue_mut().unwrap().push_back(pc);
        SimulationEvent::Stepped
    }

    /// Move the program counter forward twice and then queue the program
    /// counter onto the current queue
    fn skip_and_queue_pc(&mut self) -> SimulationEvent
    {
        self.skip_pc();

        let pc = self.pc().unwrap();
        self.current_queue_mut().unwrap().push_back(pc);
        SimulationEvent::Skipped
    }

    /// Jump the program counter by an offset and then queue the program
    /// count onto the current queue
    ///
    /// # Arguments
    /// * `offset`: amount to jump by
    fn jump_and_queue_pc(&mut self, offset: Offset) -> SimulationEvent
    {
        self.jump_pc(offset);
        
        let new_pc = self.pc().unwrap();
        self.current_queue_mut().unwrap().push_back(new_pc);
        SimulationEvent::Jumped
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

    /// Store an instruction in a specified pspace
    ///
    /// # Arguments
    /// * `pin`: programs pin, used as a lookup key
    /// * `addr`: address in the pspace to store
    /// * `instr`: instruction to store
    fn store_pspace(&mut self, pin: Pin, addr: Address, value: Offset)
        -> Result<(), ()>
    {
        if let Some(pspace) = self.pspace.get_mut(&pin) {
            let pspace_size = pspace.len();
            pspace[addr as usize % pspace_size] = value;
            Ok(())
        } else {
            Err(())
        }
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

    /// Fetch an instruction from a programs private storage
    ///
    /// # Arguments
    /// * `pin`: pin of program, used as lookup key
    /// * `addr`: address of pspace to access
    fn fetch_pspace(&self, pin: Pin, addr: Address) -> Result<Offset, ()>
    {
        if let Some(pspace) = self.pspace.get(&pin) {
            Ok(pspace[addr as usize % pspace.len()])
        } else {
            Err(())
        }
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
    fn exec_dat(&mut self) -> SimulationEvent
    {
        SimulationEvent::Terminated
    }

    /// Execute `mov` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self) -> SimulationEvent
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
    fn exec_add(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        match self.ir.op.mode {
            OpMode::A  => b.a.offset = (b.a.offset + a.a.offset) % self.size() as Offset,
            OpMode::B  => b.b.offset = (b.b.offset + a.b.offset) % self.size() as Offset,
            OpMode::BA => b.a.offset = (b.a.offset + a.b.offset) % self.size() as Offset,
            OpMode::AB => b.b.offset = (b.b.offset + a.a.offset) % self.size() as Offset,
            OpMode::F
                | OpMode::I =>
            {
                b.a.offset = (b.a.offset + a.a.offset) % self.size() as Offset;
                b.b.offset = (b.b.offset + a.b.offset) % self.size() as Offset;
            },
            OpMode::X =>
            {
                b.b.offset = (b.b.offset + a.a.offset) % self.size() as Offset;
                b.a.offset = (b.a.offset + a.b.offset) % self.size() as Offset;
            },
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
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
    fn exec_mul(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
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
    fn exec_div(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        // TODO: division by zero needs to kill the process
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
    fn exec_mod(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        // TODO: division by zero needs to kill the process
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
    fn exec_jmp(&mut self) -> SimulationEvent
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

        SimulationEvent::Jumped
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(&mut self) -> SimulationEvent
    {
        let b = self.fetch_effective_b();
        let offset = self.ir.a.offset; // TODO: needs to calculate jump offset

        let jump = match self.ir.op.mode {
            OpMode::A
                | OpMode::BA => b.a.offset == 0,
            OpMode::B
                | OpMode::AB => b.b.offset == 0,
            OpMode::F
                | OpMode::I
                | OpMode::X => b.a.offset == 0 && b.b.offset == 0,
        };

        if jump {
            self.jump_and_queue_pc(offset)
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(&mut self) -> SimulationEvent
    {
        let b = self.fetch_effective_b();
        let offset = self.ir.a.offset; // TODO: needs to calculate jump offset

        let jump = match self.ir.op.mode {
            OpMode::A
                | OpMode::BA => b.a.offset != 0,
            OpMode::B
                | OpMode::AB => b.b.offset != 0,
            OpMode::F
                | OpMode::I
                | OpMode::X => b.a.offset != 0 && b.b.offset != 0,
        };

        if jump {
            self.jump_and_queue_pc(offset)
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(&mut self) -> SimulationEvent
    {
        // predecrement the instruction before checking if its not zero
        let mut b = self.fetch_effective_b();
        match self.ir.op.mode {
            OpMode::A
                | OpMode::BA => b.a.offset -= 1,
            OpMode::B
                | OpMode::AB => b.b.offset -= 1,
            OpMode::F
                | OpMode::I
                | OpMode::X =>
            {
                b.a.offset -= 1;
                b.b.offset -= 1;
            }
        }
        self.store_effective_b(b);

        self.exec_jmn()
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(&mut self) -> SimulationEvent
    {
        if self.process_count() < self.max_processes(){
            let target = self.effective_addr_a();

            self.current_queue_mut().unwrap().push_back(target);
            self.step_and_queue_pc();
            SimulationEvent::Split
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&mut self) -> SimulationEvent
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
    fn exec_sne(&mut self) -> SimulationEvent
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
    fn exec_slt(&mut self) -> SimulationEvent
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
    fn exec_ldp(&mut self) -> SimulationEvent
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(&mut self) -> SimulationEvent
    {
        unimplemented!();
    }

    /// Execute 'nop' instruction
    fn exec_nop(&mut self) -> SimulationEvent
    {
        self.step_and_queue_pc()
    }
}

#[cfg(test)]
mod test_mars
{
    use simulation::MarsBuilder;
    use super::*;

    #[test]
    fn test_load_batch_fails_empty_vector()
    {
        let mut mars = MarsBuilder::new().build();

        let result = mars.load_batch(vec![]);

        assert_eq!(Err(LoadError::EmptyLoad), result);
    }

    #[test]
    fn test_load_suceeds()
    {
        let mut mars = MarsBuilder::new().build();
        let max_length = mars.max_length();
    
        let result = mars.load(
            0,
            None,
            &vec![Instruction::default(); max_length - 1]
            );

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn test_load_fails_program_too_long()
    {
        let mut mars = MarsBuilder::new().build();
        let max_length = mars.max_length();

        let result = mars.load(
            0,
            None, 
            &vec![Instruction::default(); max_length + 1]
            );

        assert_eq!(Err(LoadError::InvalidLength), result);
    }

    #[test]
    #[ignore]
    fn test_load_batch_load_fails_invalid_margin()
    {
    }

    #[test]
    #[ignore]
    fn test_batch_load_succeeds()
    {
    }

    #[test]
    fn test_step_errors_when_halted()
    {
        let mut mars = MarsBuilder::new().build();
        let result = mars.step();

        assert_eq!(Err(SimulationError::Halted), result);
    }

    #[test]
    fn test_dat()
    {
        let mut mars = MarsBuilder::new().build_and_load(vec![
            (0, None, &vec![Instruction::default(); 1])
            ])
            .unwrap();

        let result = mars.step();
        assert_eq!(Ok(SimulationEvent::Terminated), result);
    }

    #[test]
    fn test_step_returns_stepped_on_mov()
    {
        let prog = vec![
            Instruction {
                op: OpField {
                    code: OpCode::Mov,
                    mode: OpMode::I
                },
                a: Field {
                    offset: 0,
                    mode: AddressingMode::Direct,
                },
                b: Field {
                    offset: 1,
                    mode: AddressingMode::Direct,
                }
            },
        ];

        let mut mars = MarsBuilder::new().build_and_load(vec![
            (0, None, &prog)
            ])
            .unwrap();

        let result = mars.step();
        assert_eq!(Ok(SimulationEvent::Stepped), result);
    }
}

