
use redcode::{Instruction, OpMode, OpCode, OpField, AddressingMode, Field};
use super::Simulator;

/// Insruction that a core is loaded with by default
pub const DEFAULT_INSTRUCTION: Instruction = Instruction {
    op: OpField { mode: OpMode::I, op: OpCode::Dat },
    a:  Field   { mode: AddressingMode::Direct, offset: 0 },
    b:  Field   { mode: AddressingMode::Direct, offset: 0 },
};

// Simulator defaults (public?)
const DEFAULT_CORE_SIZE: usize = 8000;
const DEFAULT_PSPACE_SIZE: usize = 500;
const DEFAULT_MAX_CYCLES: usize = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize = 100;
const DEFAULT_MIN_DISTANCE: usize = 100;
const DEFAULT_VERSION: usize = 80; // FIXME: hmmm


/// A `Simulator` builder. Provides control over how the `Simulator` is 
/// configured
pub struct SimulatorBuilder
{
    /// Size of core's memory buffer
    core_size: usize,

    /// Size of each warrior's p-space
    pspace_size: usize,

    /// Maximum number of cycles before game is considered a draw
    max_cycles: usize,

    /// Maximum number of processes that can be in the process queue
    max_processes: usize,

    /// Maximum number of instructions a warrior can be comprised of
    max_length: usize,

    /// Minimum distance between two warriors
    min_distance: usize,

    /// Simulator Version multiplied by 100
    version: usize,
}


impl SimulatorBuilder
{
    /// Create a `SimulatorBuilder` with default parameters
    pub fn new() -> Self
    {
        SimulatorBuilder {
            core_size: DEFAULT_CORE_SIZE,
            pspace_size: DEFAULT_PSPACE_SIZE,
            max_cycles: DEFAULT_MAX_CYCLES,
            max_processes: DEFAULT_MAX_PROCESSES,
            max_length: DEFAULT_MAX_LENGTH,
            min_distance: DEFAULT_MIN_DISTANCE,
            version: DEFAULT_VERSION
        }
    }

    /// Load programs into memory and build a `Simulator`
    pub fn load(&self, programs: Vec<(usize, Vec<Instruction>)>) 
        -> Result<Simulator, ()> // TODO: add descriptive builder errors
    {
        unimplemented!();
    }

    /// Size of the `Simulator`'s memory
    pub fn core_size(&mut self, size: usize) -> &Self
    {
        self.core_size = size;
        self
    }

    /// Size of each warrior's P-space
    pub fn pspace_size(&mut self, size: usize) -> &Self
    {
        self.pspace_size = size;
        self
    }

    /// Maximum number of cycles that can elapse before a tie is declared
    pub fn max_cycles(&mut self, n: usize) -> &Self
    {
        self.max_cycles = n;
        self
    }

    /// Maximum number of processes a core can have in it's process queue
    pub fn max_processes(&mut self, n: usize) -> &Self
    {
        self.max_processes = n;
        self
    }

    /// Maximum number of instructions a warrior can contain
    pub fn max_length(&mut self, n: usize) -> &Self
    {
        self.max_length = n;
        self
    }

    /// Minimum distance between warriors
    pub fn min_distance(&mut self, n: usize) -> &Self
    {
        self.min_distance = n;
        self
    }

    /// Simulator version multiplied by 100 (e.g. version 0.8 -> 80)
    pub fn version(&mut self, version: usize) -> &Self
    {
        self.version = version;
        self
    }
}

