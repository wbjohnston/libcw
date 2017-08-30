//! Simulation runtime (aka `Core`) and tools to build a core

use std::collections::{VecDeque, HashMap};

use redcode::*;

use simulation::Event;
use simulation::Error;

pub type CoreResult<T> = Result<T, Error>;

// Core defaults (public?)
const DEFAULT_CORE_SIZE: usize     = 8000;
const DEFAULT_PSPACE_SIZE: usize   = 500;
const DEFAULT_MAX_CYCLES: usize    = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize    = 100;
const DEFAULT_MIN_DISTANCE: usize  = 100;
const DEFAULT_VERSION: usize       = 80; // FIXME: hmmm

/// Insruction that a core is loaded with by default
const DEFAULT_INSTRUCTION: Instruction = Instruction {
    op:     OpCode::Dat,
    mode:   OpMode::I,
    a:      0,
    a_mode: AddressingMode::Direct,
    b:      0,
    b_mode: AddressingMode::Direct,
};

// TODO: I think that the call structure for the simulator is all wrong
//      It leaves no access to the programs process queue, which is not good.
//      I also don't really want to add a pointer to the active process queue
//      need to think about to how organize it. Maybe pass the process queue
//      as a parameter
/// Core wars Core
#[derive(Debug, Clone)]
pub struct Core
{
    /// Core memory
    memory:        Vec<Instruction>,

    /// Current process id being run
    active_pid:    usize,

    /// Maximum of processes that can be on the process queue at any time
    max_processes: usize,

    /// Program counter for each process currently loaded into memory
    process_queue: VecDeque<(usize, VecDeque<usize>)>,

    /// Private storage space for warriors
    pspace:        HashMap<usize, Vec<Instruction>>,

    /// Core version
    version:       usize,
}

impl Core
{
    /// Step forward one cycle
    pub fn step(&mut self)
        -> CoreResult<Event>
    {
        // FIXME: this is written pretty badly
        // get active process counter
        if let Some((pid, mut q)) = self.process_queue.pop_back() {
            self.active_pid = pid;
            let pc = q.pop_back().unwrap(); 

            // fetch phase
            let i = self.fetch(pc);

            // TODO: Predecrement phase

            // execution phase
            let exec_event = match i.op {
                OpCode::Dat => self.exec_dat(),
                OpCode::Mov => self.exec_mov(&i),
                OpCode::Add => self.exec_add(&i),
                OpCode::Sub => self.exec_sub(&i),
                OpCode::Mul => self.exec_mul(&i),
                OpCode::Div => self.exec_div(&i),
                OpCode::Mod => self.exec_mod(&i),
                OpCode::Jmp => self.exec_jmp(&i),
                OpCode::Jmz => self.exec_jmz(&i),
                OpCode::Jmn => self.exec_jmn(&i),
                OpCode::Djn => self.exec_djn(&i),
                OpCode::Spl => self.exec_spl(&i),
                OpCode::Cmp => self.exec_cmp(&i),
                OpCode::Seq => self.exec_seq(&i),
                OpCode::Sne => self.exec_sne(&i),
                OpCode::Slt => self.exec_slt(&i),
                OpCode::Ldp => self.exec_ldp(&i),
                OpCode::Stp => self.exec_stp(&i),
                OpCode::Nop => self.exec_nop(),
            }?;

            // requeue process queue if there are still threads
            // TODO: process results of exec_* fns

            // TODO: PostIncrement phase

            Ok(exec_event)
        } else {
            // tried stepping after the core has terminated
            Err(Error::AlreadyTerminated)
        }
    }

    /// Fetch `Instruction` at target address
    ///
    /// # Arguments
    /// `addr`: address of `Instruction` to fetch
    ///
    /// # Return
    /// `Instruction` at `addr`
    fn fetch(&self, addr: usize) -> Instruction
    {
        let msize = self.memory.len();

        self.memory[addr % msize]
    }

    /// Fetch an mutable reference to target `Instruction`
    ///
    /// # Arguments
    /// * `addr`: address of `Instruction` to fetch
    ///
    /// # Return
    /// mutable reference to `Instruction` at `addr`
    fn fetch_mut(&mut self, addr: usize) -> &mut Instruction
    {
        let msize = self.memory.len();
        &mut self.memory[addr % msize]
    }

    /// Calculate an address considering that address calculation is done 
    /// modulo size of core
    ///
    /// # Arguments
    /// * `addr`: base address
    ///
    /// # Return
    /// address plus offset modulo core size
    fn calc_addr(&self, addr: usize, offset: isize) -> usize
    {
        let is_negative = offset < 0;
        let offset = -offset as usize;
        let msize = self.memory.len();
        
        if is_negative {
            // lower bound check    
            if addr < offset {
                msize - (offset - addr)
            } else {
                addr - offset
            }
        } else {
            // upper bound check
            if addr + offset > msize {
                msize - addr + offset
            } else {
                addr + offset
            }
        }
    }

    /////////////
    // Instruction Execution functions
    /////////////
    /// Execute `dat` instruction
    fn exec_dat(&mut self) 
        -> CoreResult<Event>
    {
        Ok(Event::Terminated(self.active_pid()))
    }

    /// Execute `mov` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_mov(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `add` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_add(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `sub` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_sub(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `mul` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_mul(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `div` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_div(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `mod` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_mod(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_jmp(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_jmz(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_jmn(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_djn(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_spl(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_cmp(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_seq(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_sne(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_slt(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_ldp(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    fn exec_stp(&mut self, i: &Instruction)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `nop` instruction
    fn exec_nop(&mut self) 
        -> CoreResult<Event>
    {
        Ok(Event::None)
    }

    /////////////
    // Data accessors
    /////////////
    /// Get immutable reference to memory
    #[inline]
    pub fn memory(&self) -> &Vec<Instruction>
    {
        &self.memory
    }

    /// Get the current process id being run
    #[inline]
    pub fn active_pid(&self) -> usize
    {
        self.active_pid
    }

    /// The number of programs currently loaded into memory
    #[inline]
    pub fn pcount(&self) -> usize
    {
        self.process_queue.len()
    }

    /// Get the number of process currently running
    #[inline]
    pub fn process_count(&self) -> usize
    {
        // count length of all local process queues in the global pqueue
        self.process_queue.iter().fold(0, |acc, &(_, ref x)| acc + x.len())
    }
}

/// Errors that can occur from invalid `CoreBuilder` configuration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuilderError
{
    /// Program is longer than the core allows
    ProgramTooLong,

    /// A provided offset would violate a constraint of the `Core`
    InvalidOffset
}

/// A `Core` builder. Provides control over how the `Core` is 
/// configured
#[derive(Debug, Clone)]
pub struct CoreBuilder
{
    /// Size of core's memory buffer
    core_size:     usize,

    /// Size of each warrior's p-space
    pspace_size:   usize,

    /// Maximum number of cycles before game is considered a draw
    max_cycles:    usize,

    /// Maximum number of processes that can be in the process queue
    max_processes: usize,

    /// Maximum number of instructions a warrior can be comprised of
    max_length:    usize,

    /// Minimum distance between two warriors
    min_distance:  usize,

    /// Core Version multiplied by 100
    version:       usize,
}

impl CoreBuilder
{
    /// Create a `CoreBuilder` with default parameters
    pub fn new() -> Self
    {
        CoreBuilder {
            core_size:     DEFAULT_CORE_SIZE,
            pspace_size:   DEFAULT_PSPACE_SIZE,
            max_cycles:    DEFAULT_MAX_CYCLES,
            max_processes: DEFAULT_MAX_PROCESSES,
            max_length:    DEFAULT_MAX_LENGTH,
            min_distance:  DEFAULT_MIN_DISTANCE,
            version:       DEFAULT_VERSION
        }
    }

    /// Load programs into memory and build a `Core`
    pub fn load(&self, programs: Vec<(usize, Program)>) 
        -> Result<Core, BuilderError>
    {
        // FIXME: this function is shit mania dot com

        // **things that happen in this function**
        // 1. Verify that the programs can fit in memory space with the
        //      correct distance
        // 2. Verify that all programs are less than `max_length`
        // 3. Load programs into memory at the correct offsets
        // 4. Add local process queue to global process queue

        // init struct data structures
        let mut mem       = vec![DEFAULT_INSTRUCTION; self.core_size];
        let mut process_q = VecDeque::new();
        let mut pspace    = HashMap::new();

        // Proposed change to all range checks
        // create iterator of all spans for programs
        // re-use that iterator to do range checks

        // sort programs by offset
        let mut sorted_programs = programs.clone();
        sorted_programs.sort_by(|a, b| a.0.cmp(&b.0));

        // Check that all programs are a valid length
        let programs_length_valid = sorted_programs.iter()
            .fold(true, |acc, &(_, ref x)| acc && x.len() < self.max_length);

        if !programs_length_valid {
            return Err(BuilderError::ProgramTooLong);
        }

        // Check if any programs are out of bounds of 
        // FIXME: this is ugly af
        let programs_are_inbound = {
            let ref last = sorted_programs[sorted_programs.len() - 1];
            let terminal_address = last.0 + last.1.len();
            terminal_address < self.core_size
        };

        if !programs_are_inbound {
            return Err(BuilderError::InvalidOffset);
        }

        // Load programs and check if all programs have enough distance 
        // between them
        let mut spans: Vec<(usize, usize)> = vec![];
        for (i, &(offset, ref program)) in sorted_programs.iter().enumerate() {
            // check margin
            if !spans.is_empty() && spans[i - 1].1 - offset < self.min_distance {
                return Err(BuilderError::InvalidOffset);
            }

            // copy program into memory
            for i in 0..programs.len() {
                mem[(i + offset) % self.core_size] = program[i];
            }

            // add program to global process queue
            let mut local_q = VecDeque::new();
            local_q.push_back(offset);
            process_q.push_back((i, local_q));
            
            // create pspace using the PID as the key
            let local_pspace = vec![DEFAULT_INSTRUCTION; self.pspace_size];
            pspace.insert(i, local_pspace);

            spans.push((offset, offset + programs.len()));
            // TODO: check wrap around distance
        }

        Ok(Core {
            memory:        mem,
            active_pid:    0,
            version:       self.version,
            max_processes: self.max_processes,
            process_queue: process_q,
            pspace:        pspace
        })
    }

    /// Size of memory
    ///
    /// # Arguments
    /// * `size`: size of memory
    ///
    /// # Return
    /// `Self`
    pub fn core_size(&mut self, size: usize) -> &Self
    {
        self.core_size = size;
        self
    }

    /// Size of each warrior's P-space
    ///
    /// # Arguments
    /// * `size`: size of memory
    ///
    /// # Return
    /// `Self`
    pub fn pspace_size(&mut self, size: usize) -> &Self
    {
        self.pspace_size = size;
        self
    }

    /// Maximum number of cycles that can elapse before a tie is declared
    ///
    /// # Arguments
    /// * `n`: number of cycles
    ///
    /// # Return
    /// `Self` 
    pub fn max_cycles(&mut self, n: usize) -> &Self
    {
        self.max_cycles = n;
        self
    }

    /// Maximum number of processes a core can have in it's process queue
    ///
    /// # Arguments
    /// * `n`: number of processes
    ///
    /// # Return
    /// `Self`
    pub fn max_processes(&mut self, n: usize) -> &Self
    {
        self.max_processes = n;
        self
    }

    /// Maximum number of instructions allowed in a program
    ///
    /// # Arguments
    /// * `n`: number of instructions
    ///
    /// # Return
    /// `Self`
    pub fn max_length(&mut self, n: usize) -> &Self
    {
        self.max_length = n;
        self
    }

    /// Minimum distance between warriors
    ///
    /// # Arguments
    /// * `n`: number of instructions
    ///
    /// # Return
    /// `Self`
    pub fn min_distance(&mut self, n: usize) -> &Self
    {
        self.min_distance = n;
        self
    }

    /// Core version multiplied by 100 (e.g. version 0.8 -> 80)
    /// # Arguments
    /// * `version`: version number
    /// # Return
    /// `Self`
    pub fn version(&mut self, version: usize) -> &Self
    {
        self.version = version;
        self
    }
}

