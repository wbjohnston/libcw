//! Utility struct for builder `Mars`s

use std::collections::{VecDeque, HashMap};

use redcode::{Instruction, Pin, Address, Program};
use simulation::Mars;

// Mars defaults
const DEFAULT_CORE_SIZE: usize     = 8000;
const DEFAULT_PSPACE_SIZE: usize   = 500;
const DEFAULT_MAX_CYCLES: usize    = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize    = 100;
const DEFAULT_MIN_DISTANCE: usize  = 100;
const DEFAULT_VERSION: usize       = 80; // FIXME: hmmm

/// Errors that can occur from invalid `MarsBuilder` configuration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuilderError
{
    /// Program is longer than the core allows
    ProgramTooLong,

    /// A provided offset would violate a constraint of the `Mars`
    InvalidOffset
}

/// A `Mars` builder. Provides control over how the `Mars` is
/// configured
#[derive(Debug, Clone)]
pub struct MarsBuilder
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

    /// Mars Version multiplied by 100
    version:       usize,
}

impl MarsBuilder
{
    /// Create a `MarsBuilder` with default parameters
    pub fn new() -> Self
    {
        MarsBuilder {
            core_size:     DEFAULT_CORE_SIZE,
            pspace_size:   DEFAULT_PSPACE_SIZE,
            max_cycles:    DEFAULT_MAX_CYCLES,
            max_processes: DEFAULT_MAX_PROCESSES,
            max_length:    DEFAULT_MAX_LENGTH,
            min_distance:  DEFAULT_MIN_DISTANCE,
            version:       DEFAULT_VERSION
        }
    }

    /// Build a core and load it with specified programs
    pub fn build_and_load(&self, programs: Vec<(Address, Option<Pin>, Program)>) 
        -> Result<Mars, ()>
    {
        let mut core = self.build();
        if programs.len() > 0 {
            core.load_batch(programs)?;
        }
        Ok(core)
    }

    /// Load programs into memory and build a `Mars`
    ///
    /// # Examples
    /// TODO
    pub fn build(&self) -> Mars
    {
        // create core resources
        let mem    = vec![Instruction::default(); self.core_size];
        let pq     = VecDeque::new();
        let pspace = HashMap::new();

        Mars {
            // Runtime data
            memory:        mem,
            cycle: 0,
            process_queue: pq,
            pspace:        pspace,
            halted:        true,
            ir:            Instruction::default(),

            // Load constraints
            max_length:    self.max_length,
            min_distance:  self.min_distance,

            // Mars information(const)
            version:       self.version,
            pspace_size:   self.pspace_size,

            // Runtime constraints
            max_processes: self.max_processes,
            max_cycles:    self.max_cycles,
        }
    }

    /// Size of memory
    ///
    /// # Examples
    /// ```
    /// use libcw::simulation::MarsBuilder;
    ///
    /// let core = MarsBuilder::new()
    ///     .core_size(80)
    ///     .build_and_load(vec![])
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
    pub fn core_size(&mut self, size: usize) -> &mut Self
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
    pub fn pspace_size(&mut self, size: usize) -> &mut Self
    {
        self.pspace_size = size;
        self
    }

    /// Maximum number of cycles that can elapse before a tie is declared
    ///
    /// # Examples
    ///
    /// ```
    /// use libcw::simulation::MarsBuilder;
    ///
    /// let core = MarsBuilder::new()
    ///     .max_cycles(100)
    ///     .build_and_load(vec![])
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
    pub fn max_cycles(&mut self, n: usize) -> &mut Self
    {
        self.max_cycles = n;
        self
    }

    /// Maximum number of processes a core can have in it's process queue
    ///
    /// # Examples
    /// ```
    /// use libcw::simulation::MarsBuilder;
    /// let core = MarsBuilder::new()
    ///     .max_processes(10)
    ///     .build_and_load(vec![])
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
    pub fn max_processes(&mut self, n: usize) -> &mut Self
    {
        self.max_processes = n;
        self
    }

    /// Maximum number of instructions allowed in a program
    ///
    /// # Examples
    /// ```
    /// use libcw::simulation::MarsBuilder;
    ///
    /// let core = MarsBuilder::new()
    ///     .max_length(100)
    ///     .build();
    ///
    /// assert_eq!(100, core.max_length());
    /// ```
    ///
    /// # Arguments
    /// * `n`: number of instructions
    ///
    /// # Return
    /// `Self`
    pub fn max_length(&mut self, n: usize) -> &mut Self
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
    pub fn min_distance(&mut self, n: usize) -> &mut Self
    {
        self.min_distance = n;
        self
    }

    /// Mars version multiplied by 100 (e.g. version 0.8 -> 80)
    ///
    /// # Arguments
    /// * `version`: version number
    ///
    /// # Return
    /// `Self`
    pub fn version(&mut self, version: usize) -> &mut Self
    {
        self.version = version;
        self
    }
}
