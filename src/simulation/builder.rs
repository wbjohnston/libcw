//! Utility struct for builder `Core`s

use std::collections::{VecDeque, HashMap};

use redcode::{Instruction, Program, Address, Pin, Pid};
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
    /// let program = vec![Instruction::default(); 10];
    ///
    /// let starting_address = 2; // program will be loaded at this addr
    /// let core = CoreBuilder::new()
    ///     .core_size(8)
    ///     .load(vec![(starting_address, None, program.clone())])
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
    pub fn load(&self, programs: Vec<(Address, Option<Pin>, Program)>) 
        -> Result<Core, BuilderError>
    {
        // create core resources
        let mut mem = vec![Instruction::default(); self.core_size];
        let mut pq  = VecDeque::new();
        let mut pspace  = HashMap::new();

        //constraint validation
        let all_valid_length = programs.iter()
            // .map(|&(_, _, ref prog)| prog.len())
            .fold(
                true,
                |acc, &(_, _, ref prog)| acc && prog.len() <= self.max_length
            );

        if !all_valid_length {
            return Err(BuilderError::ProgramTooLong);
        }

        // FIXME: compress this into a single loop
        // prepare memory
        for &(base, _, ref program) in programs.iter() {
            // copy program into memory
            for i in 0..program.len() {
                mem[base as usize + i] = program[i];
            }
        }

        // prepare pspace
        for (pid, &(_, maybe_pin, _)) in programs.iter().enumerate() {
            let pin = maybe_pin.unwrap_or(pid as Pin);
            pspace.insert(pin, vec![Instruction::default(); self.pspace_size]);
        }

        // prepare process queue
        for (pid, &(base, _, _)) in programs.iter().enumerate() {
            let mut local_pq = VecDeque::new();
            local_pq.push_front(base); 
            pq.push_front((pid as Pid, local_pq));
        }

        // this handles the case where no programs were loaded
        let (init_pid, mut init_queue) = 
            pq.pop_back().unwrap_or((0 as Pid, VecDeque::new()));

        let init_pc = init_queue.pop_back().unwrap_or(0 as Address); 

        Ok(Core {
            // Runtime data
            memory:        mem,
            current_pid:   init_pid,
            current_queue: init_queue,
            pc:            init_pc,
            process_queue: pq,
            pspace:        pspace,
            finished:      false,

            // Runtime constraints
            version:       self.version,
            max_processes: self.max_processes,
            max_cycles:    self.max_cycles,
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
    ///     .load(vec![(0, None, vec![ins; 101])]);
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
