//! Simulation runtime (aka `Core`) and tools to build a core

use std::collections::{VecDeque, HashMap};

use redcode::*;

use simulation::{Event, Error};

/// Process ID
pub type PID = usize;

pub type CoreResult<T> = Result<T, Error>;

// Core defaults (public?)
const DEFAULT_CORE_SIZE: usize     = 8000;
const DEFAULT_PSPACE_SIZE: usize   = 500;
const DEFAULT_MAX_CYCLES: usize    = 80000;
const DEFAULT_MAX_PROCESSES: usize = 8000;
const DEFAULT_MAX_LENGTH: usize    = 100;
const DEFAULT_MIN_DISTANCE: usize  = 100;
const DEFAULT_VERSION: usize       = 80; // FIXME: hmmm

// TODO: I think that the call structure for the simulator is all wrong
//      It leaves no access to the programs process queue, which is not good.
//      I also don't really want to add a pointer to the active process queue
//      need to think about to how organize it. Maybe pass the process queue
//      as a parameter
/// Core wars Core
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core
{
    /// Core memory
    memory:        Vec<Instruction>,

    /// Current process id being run
    last_pid:    Option<PID>,

    /// Maximum of processes that can be on the process queue at any time
    max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    max_cycles:    usize,

    /// Program counter for each process currently loaded into memory
    process_queue: VecDeque<(PID, VecDeque<usize>)>,

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
            self.last_pid = Some(pid);
            let pc = q.pop_back().unwrap(); 

            // fetch phase
            let i = self.fetch(pc);

            // execution phase
            let exec_event = match i.op.code {
                OpCode::Dat => self.exec_dat(),
                OpCode::Mov => self.exec_mov(i, pc),
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
                OpCode::Cmp => self.exec_cmp(),
                OpCode::Seq => self.exec_seq(),
                OpCode::Sne => self.exec_sne(),
                OpCode::Slt => self.exec_slt(),
                OpCode::Ldp => self.exec_ldp(),
                OpCode::Stp => self.exec_stp(),
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

    pub fn max_cycles(&self) -> usize
    {
        self.max_cycles
    }

    /// Fetch `Instruction` at target address
    ///
    /// # Arguments
    /// `addr`: address of `Instruction` to fetch
    ///
    /// # Return
    /// `Instruction` at `addr`
    fn fetch(&self, addr: Address) -> Instruction
    {
        self.memory[self.calc_addr(addr, 0)]
    }

    /// Fetch an mutable reference to target `Instruction`
    ///
    /// # Arguments
    /// * `addr`: address of `Instruction` to fetch
    ///
    /// # Return
    /// mutable reference to `Instruction` at `addr`
    fn fetch_mut(&mut self, addr: Address) -> &mut Instruction
    {
        let addr = self.calc_addr(addr, 0);
        &mut self.memory[addr]
    }

    /// Calculate an address considering that address calculation is done 
    /// modulo size of core
    ///
    /// # Arguments
    /// * `addr`: base address
    ///
    /// # Return
    /// address plus offset modulo core size
    fn calc_addr(&self, addr: Address, offset: Offset) 
        -> Address
    {
        unimplemented!();
    }

    fn calc_target_addr(&self, 
        addr: Address,
        offset: Offset,
        mode: AddressingMode
        ) 
        -> Address
    {
        // calculate first so we don't have to do multiple function calls
        let direct_addr = self.calc_addr(addr, offset);

        match mode {
            AddressingMode::Direct => direct_addr,

            AddressingMode::AIndirect |
                AddressingMode::AIndirectPreDecrement |
                AddressingMode::AIndirectPostIncrement => 
            {
                let indirect_offset = self.fetch(direct_addr).a.offset;    

                self.calc_addr(direct_addr, indirect_offset)
            },

            AddressingMode::BIndirect |
                AddressingMode::BIndirectPreDecrement |
                AddressingMode::BIndirectPostIncrement =>
            {
                let indirect_offset = self.fetch(direct_addr).b.offset;    
                self.calc_addr(direct_addr, indirect_offset)
            },

            AddressingMode::Immediate => panic!("This should never happen!")
        }
    }

    /////////////
    // Instruction Execution functions
    /////////////
    /// Execute `dat` instruction
    ///
    /// Supported OpModes: None
    fn exec_dat(&mut self) 
        -> CoreResult<Event>
    {
        // we can unwrap this because my the time any exec functions have been
        // called, a pid has been loaded into the `last_pid` field
        Ok(Event::Terminated(self.last_pid.unwrap()))
    }

    /// Execute `mov` instruction
    /// 
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self, i: Instruction, pc: Address)
        -> CoreResult<Event>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            // A -> A
            OpMode::A  => {
                target.a      = source.a;
            }
            // B -> B
            OpMode::B  => {
                target.b      = source.b;
            }
            // A -> B
            OpMode::AB => {
                target.b      = source.a;
            }
            // B -> A
            OpMode::BA => {
                target.a      = source.b;
            }
            // A -> B, B -> A
            OpMode::X  => {
                target.b      = source.a;
            
                target.a      = source.b;
            }
            // A -> A, B -> B
            OpMode::F  => {
                target.a      = source.a;
            
                target.b      = source.b;
            }
            // Whole instruction
            OpMode::I  => {
                *target = source;
            }
        };

        Ok(Event::None)
    }

    /// Execute `add` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `mul` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `div` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `mod` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmp(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(&mut self)
        -> CoreResult<Event>
    {
        if self.process_count() >= self.max_processes {
            Ok(Event::Split(0)) // TODO: placeholder
        } else {
            Ok(Event::None)
        }
    }

    /// Execute `cmp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_cmp(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(&mut self)
        -> CoreResult<Event>
    {
        unimplemented!();
    }

    /// Execute `nop` instruction
    ///
    /// Supported OpModes: None
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

    /// Get the last process id run
    #[inline]
    pub fn last_pid(&self) -> Option<PID>
    {
        self.last_pid
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

