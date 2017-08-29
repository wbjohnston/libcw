//! Datastructures and functions for building and simulating a redcode core

use std::collections::{VecDeque, HashMap};

use redcode::*;

pub type SimulatorResult = Result<SimulatorEvent, SimulatorError>;

// Simulator defaults (public?)
const DEFAULT_CORE_SIZE: usize     = 8000;
const DEFAULT_PSPACE_SIZE: usize   = 500;
const DEFAULT_MAX_CYCLES: usize    = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize    = 100;
const DEFAULT_MIN_DISTANCE: usize  = 100;
const DEFAULT_VERSION: usize       = 80; // FIXME: hmmm

/// Insruction that a core is loaded with by default
const DEFAULT_INSTRUCTION: Instruction = Instruction {
    op: OpField { mode: OpMode::I, op: OpCode::Dat },
    a:  Field   { mode: AddressingMode::Direct, offset: 0 },
    b:  Field   { mode: AddressingMode::Direct, offset: 0 },
};

/// Kinds of `Simulator` runtime errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulatorError
{
    /// Thrown when trying to step after the simulation has already terminated
    AlreadyTerminated
}

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulatorEvent
{
    /// All processes terminated successfully
    Finished,

    /// Game ended in a tie
    Tied,

    /// A process terminated
    Terminated(usize),

    /// Nothing happened
    None,
}

// TODO: I think that the call structure for the simulator is all wrong
//      It leaves no access to the programs process queue, which is not good.
//      I also don't really want to add a pointer to the active process queue
//      need to think about to how organize it. Maybe pass the process queue
//      as a parameter
/// Core wars Simulator
#[derive(Debug, Clone)]
pub struct Simulator
{
    /// Simulator memory
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

impl Simulator
{
    /// Step forward one cycle
    pub fn step(&mut self) -> SimulatorResult
    {
        // FIXME: this is written pretty badly
        // get active process counter
        if let Some((pid, mut q)) = self.process_queue.pop_back() {
            self.active_pid = pid;
            let pc = q.pop_back().unwrap(); 

            // fetch phase
            let i = self.memory[pc];

            // TODO: Predecrement phase

            // execution phase
            let exec_event = match i.op.op {
                OpCode::Dat => self.exec_dat(),
                OpCode::Mov => self.exec_mov(&i, &mut q),
                OpCode::Add => self.exec_add(&i, &mut q),
                OpCode::Sub => self.exec_sub(&i, &mut q),
                OpCode::Mul => self.exec_mul(&i, &mut q),
                OpCode::Div => self.exec_div(&i, &mut q),
                OpCode::Mod => self.exec_mod(&i, &mut q),
                OpCode::Jmp => self.exec_jmp(&i, &mut q),
                OpCode::Jmz => self.exec_jmz(&i, &mut q),
                OpCode::Jmn => self.exec_jmn(&i, &mut q),
                OpCode::Djn => self.exec_djn(&i, &mut q),
                OpCode::Spl => self.exec_spl(&i, &mut q),
                OpCode::Cmp => self.exec_cmp(&i, &mut q),
                OpCode::Seq => self.exec_seq(&i, &mut q),
                OpCode::Sne => self.exec_sne(&i, &mut q),
                OpCode::Slt => self.exec_slt(&i, &mut q),
                OpCode::Ldp => self.exec_ldp(&i, &mut q),
                OpCode::Stp => self.exec_stp(&i, &mut q),
                OpCode::Nop => self.exec_nop(),
            }?;

            // requeue process queue if there are still threads
            if exec_event != SimulatorEvent::Terminated(pid) {
                self.process_queue.push_front((pid, q));
            }

            // TODO: PostIncrement phase

            Ok(exec_event)
        } else {
            // tried stepping after the core has terminated
            Err(SimulatorError::AlreadyTerminated)
        }
    }

    /////////////
    // Instruction Execution functions
    /////////////
    /// Execute `dat` instruction
    fn exec_dat(&mut self) -> SimulatorResult
    {
        Ok(SimulatorEvent::Terminated(self.active_pid()))
    }

    /// Execute `mov` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_mov(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `add` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_add(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `sub` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_sub(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `mul` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_mul(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `div` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_div(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `mod` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_mod(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_jmp(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_jmz(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_jmn(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_djn(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_spl(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_cmp(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_seq(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_sne(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_slt(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_ldp(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// # Arguments
    /// * `mode`: Mode to execute instruction in
    /// * `a`: A `Field` of the `Instruction`
    /// * `b`: B `Field` of the `Instruction`
    #[allow(unused_variables)]
    fn exec_stp(&mut self, i: &Instruction, pq: &mut VecDeque<usize>)
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `nop` instruction
    fn exec_nop(&mut self) -> SimulatorResult
    {
        Ok(SimulatorEvent::None)
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

/// Errors that can occur from invalid `SimulatorBuilder` configuration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuilderError
{
    /// Program is longer than the core allows
    ProgramTooLong,

    /// A provided offset would violate a constraint of the `Simulator`
    InvalidOffset
}

/// A `Simulator` builder. Provides control over how the `Simulator` is 
/// configured
#[derive(Debug, Clone)]
pub struct SimulatorBuilder
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

    /// Simulator Version multiplied by 100
    version:       usize,
}

impl SimulatorBuilder
{
    /// Create a `SimulatorBuilder` with default parameters
    pub fn new() -> Self
    {
        SimulatorBuilder {
            core_size:     DEFAULT_CORE_SIZE,
            pspace_size:   DEFAULT_PSPACE_SIZE,
            max_cycles:    DEFAULT_MAX_CYCLES,
            max_processes: DEFAULT_MAX_PROCESSES,
            max_length:    DEFAULT_MAX_LENGTH,
            min_distance:  DEFAULT_MIN_DISTANCE,
            version:       DEFAULT_VERSION
        }
    }

    /// Load programs into memory and build a `Simulator`
    pub fn load(&self, programs: Vec<(usize, Program)>) 
        -> Result<Simulator, BuilderError>
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

        Ok(Simulator {
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

    /// Simulator version multiplied by 100 (e.g. version 0.8 -> 80)
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

