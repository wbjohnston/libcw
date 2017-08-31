//! Utility struct for builder `Core`s

use std::collections::{VecDeque, HashMap};

use redcode::{Instruction, Address, Program};
use simulation::Core;

// Core defaults
const DEFAULT_CORE_SIZE: usize     = 8000;
const DEFAULT_PSPACE_SIZE: usize   = 500;
const DEFAULT_MAX_CYCLES: usize    = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize    = 100;
const DEFAULT_MIN_DISTANCE: usize  = 100;
const DEFAULT_VERSION: usize       = 80; // FIXME: hmmm

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
    /// 
    /// # Examples
    /// ```
    /// use libcw::simulation::*;
    /// use libcw::redcode::*;
    ///
    /// // Build program
    /// let ins = Instruction {
    ///     op: OpField {
    ///         code: OpCode::Mov,
    ///         mode: OpMode::I
    ///         },
    ///     a: Field {
    ///         offset: 0,
    ///         mode: AddressingMode::Direct
    ///         },
    ///     b: Field {
    ///         offset: 0,
    ///         mode: AddressingMode::Direct
    ///         },
    ///     };
    ///
    /// let program = vec![ins; 10];
    ///
    /// let starting_address = 100; // program will be loaded at this addr
    /// let core = CoreBuilder::new()
    ///     .load(vec![(starting_address, program.clone())])
    ///     .unwrap();
    ///
    ///let (start, end) = (starting_address, starting_address + program.len());
    ///
    /// assert_eq!(
    ///     core.memory().as_slice()[start..end],
    ///     program.as_slice()
    /// );
    /// 
    /// ```
    pub fn load(&self, programs: Vec<(Address, Program)>) 
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
        let mut mem       = vec![Instruction::default(); self.core_size];
        let mut process_q = VecDeque::new();
        let mut pspace    = HashMap::new();

        // Proposed change to all range checks
        // create iterator of all spans for programs
        // re-use that iterator to do range checks

        // sort programs by offset
        let mut sorted_programs = programs.clone();
        sorted_programs.sort_by(|a, b| a.0.cmp(&b.0));

        let spans = sorted_programs.iter()
            .map(|&(addr, ref prog)| (addr, addr + prog.len()));

        // verification step
        for (i, (start, end)) in spans.enumerate() {
            // check program length
            let program_length = end - start;
            if program_length >= self.max_length {
                return Err(BuilderError::ProgramTooLong)
            }

            let program_distance = 100000; // FIXME: this is a cludge
            if program_distance <= self.min_distance {
                return Err(BuilderError::InvalidOffset)
            }

        }

        // Load programs and check if all programs have enough distance 
        // between them
        for (pid, &(start, ref program)) in sorted_programs.iter().enumerate() {

            // copy program into memory
            for i in 0..program.len() {
                mem[(i + start) % self.core_size] = program[i];
            }

            // add program to global process queue
            let mut local_q = VecDeque::new();
            local_q.push_back(start);
            process_q.push_back((pid, local_q));
            
            // create pspace using the PID as the key
            let local_pspace = vec![Instruction::default(); self.pspace_size];
            pspace.insert(pid, local_pspace);
        }


        Ok(Core {
            memory:        mem,
            last_pid:      None,
            version:       self.version,
            max_processes: self.max_processes,
            max_cycles:    self.max_cycles,
            process_queue: process_q,
            pspace:        pspace
        })
    }

    /// Size of memory
    ///
    /// # Examples
    /// ```
    /// use libcw::simulation::CoreBuilder;
    ///
    /// let core = CoreBuilder::new()
    ///     .core_size(80)
    ///     .load(vec![])
    ///     .unwrap();
    ///
    /// assert_eq!(core.size(), 80);
    /// ```
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
    /// # Examples
    /// TODO
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
    /// # Examples
    ///
    /// ```
    /// use libcw::simulation::CoreBuilder;
    ///
    /// let core = CoreBuilder::new()
    ///     .max_cycles(100)
    ///     .load(vec![])
    ///     .unwrap();
    ///
    /// assert_eq!(100, core.max_cycles());
    /// ```
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
    /// # Examples
    /// ```
    /// use libcw::simulation::CoreBuilder;
    /// let core = CoreBuilder::new()
    ///     .max_processes(10)
    ///     .load(vec![])
    ///     .unwrap();
    ///
    /// assert_eq!(10, core.max_processes());
    /// ```
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
    /// # Examples
    /// ```
    /// use libcw::simulation::{
    ///     CoreBuilder,
    ///     BuilderError,
    ///     };
    ///
    /// use libcw::redcode::{OpMode, OpCode, AddressingMode, Instruction};
    ///
    /// let ins = Instruction::default();
    /// 
    /// let core = CoreBuilder::new()
    ///     .max_length(100)
    ///     .load(vec![(0, vec![ins; 101])]);
    ///
    /// assert_eq!(Err(BuilderError::ProgramTooLong), core);
    /// ```
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
    ///
    /// # Arguments
    /// * `version`: version number
    ///
    /// # Return
    /// `Self`
    pub fn version(&mut self, version: usize) -> &Self
    {
        self.version = version;
        self
    }
}
