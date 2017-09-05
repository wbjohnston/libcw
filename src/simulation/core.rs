//! Simulation runtime (aka `Core`) and tools to build a core

// TODO: add functions for hot-loading proceses
// TODO: implement `Core::exec_stp` and `Core::exec_ldp` functions
// TODO: implement p-space PIN's

use std::collections::{VecDeque, HashMap};
use std::fmt;
use std::error::Error;

use redcode::*;

/// Process ID
pub type PID = usize;

pub type CoreResult<T> = Result<T, CoreError>;

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreEvent
{
    /// All processes terminated successfully
    Finished,

    /// Game ended in a tie
    Tied,

    /// Process split inner contains address of new pc
    Split(Address, Address),

    /// A process terminated
    Terminated(PID),

    /// A process jumped address
    Jumped(Address),

    /// Nothing happened
    Stepped(Address),
}

/// Kinds of `Simulator` runtime errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreError
{
    /// Thrown when trying to step after the simulation has already terminated
    AlreadyTerminated
}

impl fmt::Display for CoreError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "TODO") // TODO
    }
}

impl Error for CoreError
{
    fn description(&self) -> &str
    {
        match *self {
            CoreError::AlreadyTerminated => 
                "Cannot step after simulator has terminated"
        }
    }
}

/// Core wars runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core
{
    /// Core memory
    pub(super) memory:        Vec<Instruction>,

    /// Current process id being run
    pub(super) last_pid:      Option<PID>,

    /// Maximum of processes that can be on the process queue at any time
    pub(super) max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    pub(super) max_cycles:    usize,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(PID, VecDeque<Address>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<usize, Vec<Instruction>>,

    /// Core version
    pub(super) version:       usize,
}

impl Core
{
    /// Step forward one cycle
    pub fn step(&mut self) -> CoreResult<CoreEvent>
    {
        if self.process_queue.is_empty() {
            return Err(CoreError::AlreadyTerminated);
        }

        let (pid, mut q) = self.process_queue.pop_back().unwrap();
        self.last_pid = Some(pid);
        // FIXME: this is written pretty badly
        // get active process counter
        let pc       = q.pop_back().unwrap();
        let i        = self.fetch(pc).clone();
        let src_addr = self.calc_target_addr(pc, i.a);
        let trg_addr = self.calc_target_addr(pc, i.b);
        let code     = i.op.code;
        let mode     = i.op.mode;
        
        // Pre-decrement phase
        {
            // shorthand
            use self::AddressingMode::AIndirectPreDecrement as APD;
            use self::AddressingMode::BIndirectPreDecrement as BPD;

            let src = self.fetch_mut(src_addr);                

            if i.a.mode == APD || i.a.mode == BPD {
                src.a.offset -= 1;
            }

            if i.b.mode == APD || i.b.mode == BPD {
                src.b.offset -= 1;
            }
        }
        
        let exec_event = self.execute(i)?;

        // Post increment phase
        {
            use self::AddressingMode::AIndirectPostIncrement as APD;
            use self::AddressingMode::BIndirectPostIncrement as BPD;

            let src = self.fetch_mut(src_addr);                

            if i.a.mode == APD || i.a.mode == BPD { // check a
                src.a.offset += 1;
            }

            if i.b.mode == APD || i.b.mode == BPD { // check b
                src.b.offset += 1;
            }
        }

        match exec_event {
            CoreEvent::Stepped(pc) => {
                q.push_front(pc);
                self.process_queue.push_front((pid, q));
            },
            CoreEvent::Split(pc1, pc2) => {
                q.push_front(pc1);
                q.push_front(pc2);
                self.process_queue.push_front((pid, q));
            },
            _ => {}
        };  

        // requeue process queue if there are still threads
        // TODO: process results of exec_* fns
        Ok(exec_event)
    }

    /// Execute an instruction on the core
    pub fn execute(&mut self, instr: Instruction) -> CoreResult<CoreEvent>
    {
        unimplemented!();
    }

    /// Get the program counters for all processes
    pub fn pcs(&self) -> Vec<Address>
    {
        unimplemented!();
    }

    /// Get all `PID`s that are currently active
    pub fn pids(&self) -> Vec<PID>
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
    #[inline]
    pub fn memory(&self) -> &[Instruction]
    {
        &self.memory.as_slice()
    }

    /// Get the last process id run
    #[inline]
    pub fn last_pid(&self) -> Option<PID>
    {
        self.last_pid
    }

    /// Get the number of processes currently running
    #[inline]
    pub fn process_count(&self) -> usize
    {
        // count length of all local process queues in the global pqueue
        self.process_queue.iter().fold(0, |acc, &(_, ref x)| acc + x.len())
    }

    /// Fetch `Instruction` at target address
    ///
    /// # Arguments
    /// `addr`: address of `Instruction` to fetch
    fn fetch(&self, addr: Address) -> &Instruction
    {
        &self.memory[addr as usize]
    }

    /// Fetch an mutable reference to target `Instruction`
    ///
    /// # Arguments
    /// * `addr`: address of `Instruction` to fetch
    fn fetch_mut(&mut self, addr: Address) -> &mut Instruction
    {
        let addr = addr % self.size();
        &mut self.memory[addr as usize]
    }

    /// Calculate an address plus an offset taking core size into account
    ///
    /// # Arguments
    /// * `base`: base address
    /// * `offst`: offset of base to calculate
    fn calc_addr(&self, base: Address, offset: Offset) -> Address
    {
        if offset < 0 { // positive or negative
            let offset = (-offset) as usize;

            if offset > base { // underflow
                 self.size() - (offset - base)
            } else {
                base - offset % self.size()
            }
        } else {
            let offset = offset as usize;

            if base + offset > self.size() { // overflow
                0
            } else {
                base + offset
            }
        }
    }

    /// Calculate an address taking into account indirection
    ///
    /// # Arguments
    /// * `base`: base address
    /// * `field`: instruction field that contains addressing data
    fn calc_target_addr(&self, base: Address, field: Field) -> Address
    {
        // unimplemented!("This probably doesn't work right now");
        // calculate first so we don't have to do multiple function calls
        let direct = self.calc_addr(base, field.offset);

        match field.mode {
            AddressingMode::Direct => direct,
            AddressingMode::AIndirect
                | AddressingMode::AIndirectPreDecrement
                | AddressingMode::AIndirectPostIncrement => {
                let indirect = self.fetch(direct).a.offset;    
                self.calc_addr(direct, indirect)
            },
            AddressingMode::BIndirect 
                | AddressingMode::BIndirectPreDecrement
                | AddressingMode::BIndirectPostIncrement =>
            {
                let indirect = self.fetch(direct).b.offset;    
                self.calc_addr(direct, indirect)
            },
            AddressingMode::Immediate => unreachable!() 
        }
    }

    // TODO: need to move current process queue and global queue to 
    // a field in the `Core` struct and make all execution methods mutable

    /// Execute `mov` instruction
    /// 
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a = src.a,
            OpMode::B  => trg.b = src.b,
            OpMode::AB => trg.b = src.a,
            OpMode::BA => trg.a = src.b,
            OpMode::X  => {
                trg.b = src.a;
                trg.a = src.b;
            }
            OpMode::F  => {
                trg.a = src.a;
                trg.b = src.b;
            }
            OpMode::I => *trg = *src,
        };
    }

    /// Execute `add` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a.offset += src.a.offset,
            OpMode::B  => trg.b.offset += src.b.offset,
            OpMode::AB => trg.b.offset += src.a.offset,
            OpMode::BA => trg.a.offset += src.b.offset,
            OpMode::X  => {
                trg.b.offset += src.a.offset;
                trg.a.offset += src.b.offset;
            }
            OpMode::I
                | OpMode::F => {
                trg.a.offset += src.a.offset;
                trg.b.offset += src.b.offset;
            }
        };
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a.offset -= src.a.offset,
            OpMode::B  => trg.b.offset -= src.b.offset,
            OpMode::AB => trg.b.offset -= src.a.offset,
            OpMode::BA => trg.a.offset -= src.b.offset,
            OpMode::X  => {
                trg.b.offset -= src.a.offset;
                trg.a.offset -= src.b.offset;
            }
            OpMode::I
                | OpMode::F => {
                trg.a.offset -= src.a.offset;
                trg.b.offset -= src.b.offset;
            }
        };
    }

    /// Execute `mul` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a.offset *= src.a.offset,
            OpMode::B  => trg.b.offset *= src.b.offset,
            OpMode::AB => trg.b.offset *= src.a.offset,
            OpMode::BA => trg.a.offset *= src.b.offset,
            OpMode::X  => {
                trg.b.offset *= src.a.offset;
                trg.a.offset *= src.b.offset;
            }
            OpMode::I
                | OpMode::F => {
                trg.a.offset *= src.a.offset;
                trg.b.offset *= src.b.offset;
            }
        };
    }

    /// Execute `div` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a.offset /= src.a.offset,
            OpMode::B  => trg.b.offset /= src.b.offset,
            OpMode::AB => trg.b.offset /= src.a.offset,
            OpMode::BA => trg.a.offset /= src.b.offset,
            OpMode::X  => {
                trg.b.offset /= src.a.offset;
                trg.a.offset /= src.b.offset;
            }
            OpMode::I
                | OpMode::F => {
                trg.a.offset /= src.a.offset;
                trg.b.offset /= src.b.offset;
            }
        };
    }

    /// Execute `mod` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(mode: OpMode, src: &Instruction, trg: &mut Instruction)
    {
        match mode {
            OpMode::A  => trg.a.offset %= src.a.offset,
            OpMode::B  => trg.b.offset %= src.b.offset,
            OpMode::AB => trg.b.offset %= src.a.offset,
            OpMode::BA => trg.a.offset %= src.b.offset,
            OpMode::X  => {
                trg.b.offset %= src.a.offset;
                trg.a.offset %= src.b.offset;
            }
            OpMode::I
                | OpMode::F => {
                trg.a.offset %= src.a.offset;
                trg.b.offset %= src.b.offset;
            }
        };
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    #[allow(dead_code, unused_variables)]
    fn exec_jmp(mode: OpMode, src: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    #[allow(dead_code, unused_variables)]
    fn exec_jmz(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    #[allow(dead_code, unused_variables)]
    fn exec_jmn(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    #[allow(dead_code, unused_variables)]
    fn exec_djn(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    #[allow(dead_code, unused_variables)]
    fn exec_spl(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_cmp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_seq(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_sne(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_slt(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_ldp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    #[allow(dead_code, unused_variables)]
    fn exec_stp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }
}

