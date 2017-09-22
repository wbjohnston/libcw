
use std::collections::{VecDeque, HashMap};

use redcode::types::*;
use redcode::traits::Instruction;

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
    MaxCyclesReached,

    /// Process split inner contains address of new pc
    Split,

    /// A process terminated
    Terminated,

    /// The Mars halted
    Halted,

    /// A process jumped address
    Jumped,

    /// Skipped happens in all `Skip if ...` instructions
    Skipped,

    /// Nothing happened
    Stepped,
}

/// Core wars runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mars<T>
    where T: Instruction
{
    /// Mars memory
    pub(super) memory:        Vec<T>,

    /// Instruction register
    pub(super) ir:            T,

    pub(super) pid:           Pid,

    pub(super) pc:            Address,

    /// Current numbered cycle core is executing
    pub(super) cycle:         usize,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(Pid, VecDeque<Address>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<Pin, Vec<Value>>,

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

impl<T> Mars<T>
where T: Instruction
{
    // TODO: add generic program type

    /// Step forward one cycle
    pub fn step(&mut self) -> SimulationResult<SimulationEvent>
    {
        if self.halted() { // can't step after the core is halted
            return Err(SimulationError::Halted);
        }

        else if self.cycle() >= self.max_cycles() {
            self.halted = true;
            return Ok(SimulationEvent::MaxCyclesReached)
        }

        let pc = self.pc();

        // Fetch instruction
        self.ir = self.fetch(pc);
        let (a_mode, b_mode) = (self.ir.a_mode(), self.ir.b_mode());

        // PostIncrement phase
        let predecrement = a_mode == AddressingMode::AIndirectPreDecrement ||
            a_mode == AddressingMode::BIndirectPreDecrement ||
            b_mode == AddressingMode::AIndirectPreDecrement ||
            b_mode == AddressingMode::BIndirectPreDecrement;

        // Preincrement phase
        if predecrement {
            // fetch direct target
            let a_addr = self.calc_addr_offset(pc, self.ir.a());
            let b_addr = self.calc_addr_offset(pc, self.ir.b());
            let mut a = self.fetch(a_addr);
            let mut b = self.fetch(b_addr);

            let (a_a, a_b) = (a.a(), a.b());
            let (b_a, b_b) = (b.a(), b.b());

            // FIXME: combine these into a single match statement
            match a_mode {
                AddressingMode::AIndirectPreDecrement => { a.set_a(a_a + 1); }
                AddressingMode::BIndirectPreDecrement => { a.set_b(a_b + 1); }
                _ => {}
            };

            match b_mode {
                AddressingMode::AIndirectPreDecrement => { b.set_a(b_a + 1); }
                AddressingMode::BIndirectPreDecrement => { b.set_b(b_b + 1); }
                _ => {}
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
            let a_addr = self.calc_addr_offset(pc, self.ir.a());
            let b_addr = self.calc_addr_offset(pc, self.ir.b());
            let mut a = self.fetch(a_addr);
            let mut b = self.fetch(b_addr);

            let (a_a, a_b) = (a.a(), a.b());
            let (b_a, b_b) = (b.a(), b.b());

            // FIXME: combine these into a single match statement
            match a_mode {
                AddressingMode::AIndirectPreDecrement => { a.set_a(a_a + 1); }
                AddressingMode::BIndirectPreDecrement => { a.set_b(a_b + 1); }
                _ => {}
            };

            match b_mode {
                AddressingMode::AIndirectPreDecrement => { b.set_a(b_a + 1); }
                AddressingMode::BIndirectPreDecrement => { b.set_b(b_b + 1); }
                _ => {}
            };
            // store result
            self.store(a_addr, a);
            self.store(b_addr, b);
        }

        // check if there are any more process queues running on the core
        let (pid, q) = self.process_queue.pop_front().unwrap();
        if !q.is_empty() {
            self.process_queue.push_back((pid, q));
        }

        // If no there are no processes left
        if self.process_queue.is_empty() {
            Ok(self.halt())
        } else {
            // Fetch new queue
            let &mut(curr_pid, ref mut curr_q) = self.process_queue.front_mut().unwrap();
            println!("{:?}", curr_q);
            self.pid = curr_pid;
            self.pc = curr_q.pop_front().unwrap();
            self.cycle += 1;
            Ok(exec_event)
        }
    }

    /// Has the core finished its execution. This can mean either a tie has
    /// occurred or a warrior has emerged victoriors
    pub fn halted(&self) -> bool
    {
        self.halted
    }

    /// Halt the Mars
    fn halt(&mut self) -> SimulationEvent
    {
        self.halted = true;
        SimulationEvent::Halted
    }

    /// Reset the Mars's memory and the process queue
    pub fn reset(&mut self)
    {
        // reset memory
        for e in self.memory.iter_mut() {
            *e = Default::default();
        }

        self.process_queue.clear();

        self.cycle         = 0;
        self.ir            = Default::default();
        self.halted        = true;
    }

    /// Reset the Mar's memory, process queue, AND P-space
    pub fn reset_hard(&mut self)
    {
        self.pspace.clear();
        self.reset();
    }

    /// Load mutliple programs into the Mars, checking their spacing and their
    /// length
    /// # Arguments
    /// * `programs`: programs and load information loaded in a tuple, cannot
    ///     be empty
    /// # Return
    /// `Ok(())` if the load was successful, otherwise an error with the 
    ///     corresponding `SimulationError`
    pub fn load_batch(&mut self, programs: Vec<(Address, Option<Pin>, &Vec<T>)>)
        -> LoadResult<()>
    {
        // TODO: validate margin
        // TODO: correct addresses that are out of bounds by modulo-ing them
        if programs.is_empty() {
            return Err(LoadError::EmptyLoad);
        }

        let valid_margin = true; // TODO

        if valid_margin {
            // load each program
            for &(dest, maybe_pin, ref prog) in programs.iter() {
                let pin = maybe_pin.unwrap_or(self.process_count() as Pid);

                let cycle_memory_iter = (0..self.size())
                    .cycle()
                    .skip(dest as usize)
                    .take(prog.len())
                    .enumerate();

                // copy program into memory
                for (i, j) in cycle_memory_iter {
                    self.memory[j] = prog[i].clone();
                }

                self.pspace.insert(pin, vec![0; self.pspace_size]);

                let mut q = VecDeque::new();
                q.push_front(dest);
                self.process_queue.push_front((pin, q));
            }

            self.halted = false;

            let &mut (curr_pid, ref mut curr_q) = self.process_queue.front_mut()
                .unwrap();

            self.pc = curr_q.pop_front().unwrap();
            self.pid = curr_pid;
                
            Ok(())
        } else {
            Err(LoadError::InvalidDistance)
        }
    }

    /// Get `Pid` currently executing on the core
    ///
    /// # Panics
    /// * Panics is the process queue is empty
    pub fn pc(&self) -> Address
    {
        self.pc
    }

    /// Get the program counters for all processes
    pub fn pcs(&self) -> Vec<Address>
    {
        let mut pcs = vec![self.pc()];

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
    pub fn pid(&self) -> Pid
    {
        self.pid
    }

    /// Get all `Pid`s that are currently active in the order they will be 
    /// executing
    pub fn pids(&self) -> Vec<Pid>
    {
        let mut pids = vec![self.pid()];
        pids.extend(self.process_queue.iter().map(|&(pid, _)| pid));
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
    pub fn memory(&self) -> &[T]
    {
        self.memory.as_slice()
    }

    /// Get an immutable reference to private storage
    pub fn pspace(&self) -> &HashMap<Pin, Vec<Value>>
    {
        &self.pspace
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
        match self.ir.op() {
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

    ////////////////////////////////////////////////////////////////////////////
    // Address resolution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Calculate the address after adding an offset
    ///
    /// # Arguments
    /// * `base`: base address
    /// * `offset`: distance from base to calculate
    #[inline]
    fn calc_addr_offset(&self, base: Address, offset: Value) -> Address
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
        let (mode, offset) = if use_a_field { 
            (self.ir.a_mode(), self.ir.a())
        } else {
            (self.ir.b_mode(), self.ir.b())
        };

        let pc = self.pc();

        let direct = self.fetch(self.calc_addr_offset(pc, offset));

        match mode {
            Immediate => pc,
            Direct => self.calc_addr_offset(pc, offset),
            AIndirect
                | AIndirectPreDecrement
                | AIndirectPostIncrement =>
                self.calc_addr_offset(pc, direct.a() + offset),
            BIndirect
                | BIndirectPreDecrement
                | BIndirectPostIncrement =>
                self.calc_addr_offset(pc, direct.b() + offset),
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
        let pc = self.pc();
        self.pc = (pc + 1) % self.size() as Address;
        SimulationEvent::Stepped
    }

    /// Move the program counter forward twice
    fn skip_pc(&mut self) -> SimulationEvent
    {
        let pc = self.pc();
        // TODO: Holy shit this is uuugggglllllyyyy
        self.pc = (pc + 2) % self.size() as Address;
        SimulationEvent::Skipped
    }

    /// Jump the program counter by an offset
    ///
    /// # Arguments
    /// * `offset`: amount to jump
    fn jump_pc(&mut self, offset: Value) -> SimulationEvent
    {
        let pc = self.pc();
        self.pc = self.calc_addr_offset(pc, offset);
        SimulationEvent::Jumped
    }

    /// Move the program counter forward by one and then queue the program
    /// counter onto the current queue
    fn step_and_queue_pc(&mut self) -> SimulationEvent
    {
        self.step_pc();

        let pc = self.pc();
        self.current_queue_mut().unwrap().push_back(pc);
        SimulationEvent::Stepped
    }

    /// Move the program counter forward twice and then queue the program
    /// counter onto the current queue
    fn skip_and_queue_pc(&mut self) -> SimulationEvent
    {
        self.skip_pc();

        let pc = self.pc();
        self.current_queue_mut().unwrap().push_back(pc);
        SimulationEvent::Skipped
    }

    /// Jump the program counter by an offset and then queue the program
    /// count onto the current queue
    ///
    /// # Arguments
    /// * `offset`: amount to jump by
    fn jump_and_queue_pc(&mut self, offset: Value) -> SimulationEvent
    {
        self.jump_pc(offset);
        
        // remove old pc
        let pc = self.pc();
        self.current_queue_mut().unwrap().push_back(pc);
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
    fn store(&mut self, addr: Address, instr: T)
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
    fn store_pspace(&mut self, pin: Pin, addr: Address, value: Value)
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
    fn store_effective_a(&mut self, instr: T)
    {
        let eff_addr = self.effective_addr_a();
        self.store(eff_addr, instr)
    }

    /// Store an `Instruction` into the memory location pointed at by the B
    /// field of the instruction loaded into the instruction register
    ///
    /// # Arguments
    /// * `instr`: `Instruction` to store
    fn store_effective_b(&mut self, instr: T)
    {
        let eff_addr = self.effective_addr_b();
        self.store(eff_addr, instr)
    }

    /// Fetch copy of instruction in memory
    ///
    /// # Arguments
    /// * `addr`: adress to fetch
    fn fetch(&self, addr: Address) -> T
    {
        self.memory[addr as usize % self.size()].clone()
    }

    /// Fetch an instruction from a programs private storage
    ///
    /// # Arguments
    /// * `pin`: pin of program, used as lookup key
    /// * `addr`: address of pspace to access
    fn fetch_pspace(&self, pin: Pin, addr: Address) -> Result<Value, ()>
    {
        if let Some(pspace) = self.pspace.get(&pin) {
            Ok(pspace[addr as usize % pspace.len()])
        } else {
            Err(())
        }
    }

    /// Fetch copy of instruction pointed at by the A field of the instruction
    /// loaded into the instruction register
    fn fetch_effective_a(&self) -> T
    {
        self.fetch(self.effective_addr_a())
    }

    /// Fetch copy of instruction pointed at by the B field of the instruction
    /// loaded into the instruction register
    fn fetch_effective_b(&self) -> T
    {
        self.fetch(self.effective_addr_b())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Instruction execution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Execute `dat` instruction
    ///
    /// Supported Modifiers: None
    fn exec_dat(&mut self) -> SimulationEvent
    {
        let _ = self.current_queue_mut().unwrap().pop_front();
        SimulationEvent::Terminated
    }

    /// Execute `mov` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self) -> SimulationEvent
    {
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A => {b.set_a(a_a);},
            Modifier::B => {b.set_b(a_b);},
            Modifier::AB => {b.set_a(a_b);},
            Modifier::BA => {b.set_b(a_a);},
            Modifier::F =>
            {
                b.set_a(a_a);
                b.set_b(a_b);
            },
            Modifier::X =>
            {
                b.set_a(a_b);
                b.set_b(a_a);
            },
            Modifier::I => b = a
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `add` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A  => { b.set_a((b_a + a_a) % self.size() as Value); }
            Modifier::B  => { b.set_b((b_b + a_b) % self.size() as Value); }
            Modifier::BA => { b.set_a((b_a + a_b) % self.size() as Value); }
            Modifier::AB => { b.set_b((b_b + a_a) % self.size() as Value); }
            Modifier::F
                | Modifier::I =>
            {
                b.set_a((b_a + a_a) % self.size() as Value);
                b.set_b((b_b + a_b) % self.size() as Value);
            }
            Modifier::X =>
            {
                b.set_b((b_b + a_a) % self.size() as Value);
                b.set_a((b_a + a_b) % self.size() as Value);
            }
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `sub` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A  => { b.set_a((b_a - a_a) % self.size() as Value); }
            Modifier::B  => { b.set_b((b_b - a_b) % self.size() as Value); }
            Modifier::BA => { b.set_a((b_a - a_b) % self.size() as Value); }
            Modifier::AB => { b.set_b((b_b - a_a) % self.size() as Value); }
            Modifier::F
                | Modifier::I =>
            {
                b.set_a((b_a - a_a) % self.size() as Value);
                b.set_b((b_b - a_b) % self.size() as Value);
            }
            Modifier::X =>
            {
                b.set_b((b_b - a_a) % self.size() as Value);
                b.set_a((b_a - a_b) % self.size() as Value);
            }
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `mul` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A  => { b.set_a((b_a * a_a) % self.size() as Value); }
            Modifier::B  => { b.set_b((b_b * a_b) % self.size() as Value); }
            Modifier::BA => { b.set_a((b_a * a_b) % self.size() as Value); }
            Modifier::AB => { b.set_b((b_b * a_a) % self.size() as Value); }
            Modifier::F
                | Modifier::I =>
            {
                b.set_a((b_a * a_a) % self.size() as Value);
                b.set_b((b_b * a_b) % self.size() as Value);
            }
            Modifier::X =>
            {
                b.set_b((b_b * a_a) % self.size() as Value);
                b.set_a((b_a * a_b) % self.size() as Value);
            }
        }

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `div` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        // TODO: division by zero needs to kill the process
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A  => { b.set_a((b_a / a_a) % self.size() as Value); }
            Modifier::B  => { b.set_b((b_b / a_b) % self.size() as Value); }
            Modifier::BA => { b.set_a((b_a / a_b) % self.size() as Value); }
            Modifier::AB => { b.set_b((b_b / a_a) % self.size() as Value); }
            Modifier::F
                | Modifier::I =>
            {
                b.set_a((b_a / a_a) % self.size() as Value);
                b.set_b((b_b / a_b) % self.size() as Value);
            }
            Modifier::X =>
            {
                b.set_b((b_b / a_a) % self.size() as Value);
                b.set_a((b_a / a_b) % self.size() as Value);
            }
        };

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `mod` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(&mut self) -> SimulationEvent
    {
        // TODO: math needs to be done modulo core size
        // TODO: division by zero needs to kill the process
        let a     = self.fetch_effective_a();
        let mut b = self.fetch_effective_b();

        let (a_a, a_b) = (a.a(), a.b());
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A  => { b.set_a((b_a % a_a) % self.size() as Value); }
            Modifier::B  => { b.set_b((b_b % a_b) % self.size() as Value); }
            Modifier::BA => { b.set_a((b_a % a_b) % self.size() as Value); }
            Modifier::AB => { b.set_b((b_b % a_a) % self.size() as Value); }
            Modifier::F
                | Modifier::I =>
            {
                b.set_a((b_a % a_a) % self.size() as Value);
                b.set_b((b_b % a_b) % self.size() as Value);
            }
            Modifier::X =>
            {
                b.set_b((b_b % a_a) % self.size() as Value);
                b.set_a((b_a % a_b) % self.size() as Value);
            }
        };

        self.store_effective_b(b);
        self.step_and_queue_pc()
    }

    /// Execute `jmp` instruction
    ///
    /// Supported Modifiers: `B`
    fn exec_jmp(&mut self) -> SimulationEvent
    {
        match self.ir.a_mode() {
            AddressingMode::Immediate
                | AddressingMode::Direct =>
            {
                let offset = self.ir.a();
                self.jump_and_queue_pc(offset);
            }
            // TODO
            _ => unimplemented!()
        };

        SimulationEvent::Jumped
    }

    /// Execute `jmz` instruction
    ///
    /// Supported Modifiers: `B`
    fn exec_jmz(&mut self) -> SimulationEvent
    {
        let b = self.fetch_effective_b();
        let offset = self.ir.a(); // TODO: needs to calculate jump offset

        let jump = match self.ir.modifier() {
            Modifier::A
                | Modifier::BA => b.a() == 0,
            Modifier::B
                | Modifier::AB => b.b() == 0,
            Modifier::F
                | Modifier::I
                | Modifier::X => b.a() == 0 && b.b() == 0,
        };

        if jump {
            self.jump_and_queue_pc(offset)
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `jmn` instruction
    ///
    /// Supported Modifiers: `B`
    fn exec_jmn(&mut self) -> SimulationEvent
    {
        let b = self.fetch_effective_b();
        let offset = self.ir.a(); // TODO: needs to calculate jump offset

        let jump = match self.ir.modifier() {
            Modifier::A
                | Modifier::BA => b.a() != 0,
            Modifier::B
                | Modifier::AB => b.b() != 0,
            Modifier::F
                | Modifier::I
                | Modifier::X => b.a() != 0 && b.b() != 0,
        };

        if jump {
            self.jump_and_queue_pc(offset)
        } else {
            self.step_and_queue_pc()
        }
    }

    /// Execute `djn` instruction
    ///
    /// Supported Modifiers: `B`
    fn exec_djn(&mut self) -> SimulationEvent
    {
        // predecrement the instruction before checking if its not zero
        let mut b = self.fetch_effective_b();
        let (b_a, b_b) = (b.a(), b.b());

        match self.ir.modifier() {
            Modifier::A
                | Modifier::BA => { b.set_a(b_a - 1); },
            Modifier::B
                | Modifier::AB => { b.set_b(b_b - 1); },
            Modifier::F
                | Modifier::I
                | Modifier::X =>
            {
                b.set_a(b_a - 1);
                b.set_b(b_b - 1);
            }
        };
        self.store_effective_b(b);

        self.exec_jmn()
    }

    /// Execute `spl` instruction
    ///
    /// Supported Modifiers: `B`
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
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&mut self) -> SimulationEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.modifier() {
            Modifier::A       => a.a() == b.b(),
            Modifier::B       => a.b() == b.b(),
            Modifier::BA      => a.a() == b.b(),
            Modifier::AB      => a.b() == b.a(),
            Modifier::X       => a.b() == b.a() &&
                                 a.a() == b.b(),
            Modifier::F
                | Modifier::I => a.a() == b.a() &&
                                 a.b() == b.b(),
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `sne` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(&mut self) -> SimulationEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.modifier() {
            Modifier::A       => a.a() != b.b(),
            Modifier::B       => a.b() != b.b(),
            Modifier::BA      => a.a() != b.b(),
            Modifier::AB      => a.b() != b.a(),
            Modifier::X       => a.b() != b.a() &&
                                 a.a() != b.b(),
            Modifier::F
                | Modifier::I => a.a() != b.a() &&
                                 a.b() != b.b(),
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `slt` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(&mut self) -> SimulationEvent
    {
        let a = self.fetch_effective_a();
        let b = self.fetch_effective_b();

        let skip = match self.ir.modifier() {
            Modifier::A       => a.a() < b.b(),
            Modifier::B       => a.b() < b.b(),
            Modifier::BA      => a.a() < b.b(),
            Modifier::AB      => a.b() < b.a(),
            Modifier::X       => a.b() < b.a() &&
                                 a.a() < b.b(),
            Modifier::F
                | Modifier::I => a.a() < b.a() &&
                                 a.b() < b.b(),
        };

        if skip { self.skip_and_queue_pc() } else { self.step_and_queue_pc() }
    }

    /// Execute `ldp` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(&mut self) -> SimulationEvent
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported Modifiers: `A` `B` `AB` `BA` `X` `F` `I`
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
        assert_eq!(
            Err(LoadError::EmptyLoad),
            mars.load_batch(vec![])
            );
    }

    #[test]
    fn test_load_suceeds()
    {
        let mut mars = MarsBuilder::new().build();
        let max_length = mars.max_length();
        let prog = vec![Default::default(); max_length - 1];

        assert_eq!(Ok(()), mars.load(0, None, &prog));
    }

    #[test]
    fn test_load_fails_program_too_long()
    {
        let mut mars = MarsBuilder::new().build();
        let max_length = mars.max_length();
        let prog = vec![Default::default(); max_length + 1];

        assert_eq!(
            Err(LoadError::InvalidLength),
            mars.load(0, None, &prog)
            );
    }

    #[test]
    #[ignore]
    fn test_load_batch_load_fails_invalid_distance()
    {
        let mut mars = MarsBuilder::new()
            .min_distance(10)
            .build();

        let useless_program = vec![Default::default(); 1];

        // intentionally load the programs with invalid spacings
        let result = mars.load_batch(vec![
            (0, None, &useless_program),
            (1, None, &useless_program),
        ]);
        
        assert_eq!(Err(LoadError::InvalidDistance), result);
    }

    #[test]
    fn test_load_succeeds_on_boundary()
    {
        let mut mars = MarsBuilder::new()
            .size(16)
            .build();
        
        // transform the instruction so we can recognize it in memory
        let mut program = vec![Default::default(); 4];
        for (i, e) in program.iter_mut().enumerate() {
            e.op.code = OpCode::Mov;
            e.a() = i as Value;
        }

        let result = mars.load(14, None, &program);

        assert_eq!(Ok(()), result);
        assert_eq!(program[0], mars.memory()[14]);
        assert_eq!(program[1], mars.memory()[15]);
        assert_eq!(program[2], mars.memory()[0]);
        assert_eq!(program[3], mars.memory()[1]);
    }

    #[test]
    fn test_batch_load_succeeds()
    {
        let mut mars = MarsBuilder::new()
            .min_distance(10)
            .max_length(10)
            .build();

        let useless_program = vec![Default::default(); 9];

        // intentionally load the programs with invalid spacings
        let result = mars.load_batch(vec![
            (0, None, &useless_program),
            (11, None, &useless_program),
        ]);
        
        assert_eq!(Ok(()), result);
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
            (0, None, &vec![Default::default(); 1])
            ])
            .unwrap();

        let result = mars.step();
        assert_eq!(Ok(SimulationEvent::Halted), result);
        assert_eq!(true, mars.halted());
    }

    #[test]
    fn test_mov()
    {
        let prog = vec![
            Instruction {
                op: OpField {
                    code: OpCode::Mov,
                    mode: Modifier::I
                },
                a: Field {
                    value: 0,
                    mode: AddressingMode::Direct,
                },
                b: Field {
                    value: 1,
                    mode: AddressingMode::Direct,
                }
            },
        ];

        let mut mars = MarsBuilder::new().build_and_load(vec![
            (0, None, &prog)
            ])
            .unwrap();

        let init_pc    = mars.pc();
        let init_cycle = mars.cycle();

        assert_eq!(Ok(SimulationEvent::Stepped), mars.step());
        assert_eq!(init_pc + 1,                  mars.pc());
        assert_eq!(init_cycle + 1,               mars.cycle());
    }

    #[test]
    // #[ignore]
    fn test_spl_cant_create_more_than_max_processes()
    {
        // splitter program, infinitely creates imps
        let prog = vec![
            Instruction {
                op: OpField {
                    code: OpCode::Spl,
                    mode: Modifier::I
                },
                a: Field {
                    value: 2,
                    mode: AddressingMode::Direct
                },
                b: Field {
                    value: 1,
                    mode: AddressingMode::Direct
                }
            },
            Instruction {
                op: OpField {
                    code: OpCode::Jmp,
                    mode: Modifier::I
                },
                a: Field {
                    value: -1,
                    mode: AddressingMode::Direct
                },
                b: Field {
                    value: 1,
                    mode: AddressingMode::Direct
                }
            },
            Instruction {
                op: OpField {
                    code: OpCode::Mov,
                    mode: Modifier::I
                },
                a: Field {
                    value: 0,
                    mode: AddressingMode::Direct
                },
                b: Field {
                    value: 1,
                    mode: AddressingMode::Direct
                }
            },
        ];

        let mut mars = MarsBuilder::new()
            .max_processes(10)
            .build_and_load(vec![(0, None, &prog)])
            .unwrap();

        assert_eq!(Ok(SimulationEvent::Split), mars.step());

        // run the simulation until it halts because cycles have been exauste
        while !mars.halted() {
            let _ = mars.step();
        }
        
        assert_eq!(10, mars.process_count());
    }
}

