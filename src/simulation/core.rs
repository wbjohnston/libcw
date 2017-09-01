//! Simulation runtime (aka `Core`) and tools to build a core

// TODO: add functions for hot-loading proceses
// TODO: implement `Core::exec_stp` and `Core::exec_ldp` functions

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
    pub(super) last_pid:    Option<PID>,

    /// Maximum of processes that can be on the process queue at any time
    pub(super) max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    pub(super) max_cycles:    usize,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(PID, VecDeque<usize>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<usize, Vec<Instruction>>,

    /// Core version
    pub(super) version:       usize,
}

impl Core
{
    /// Step forward one cycle
    pub fn step(&mut self)
        -> CoreResult<CoreEvent>
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
                OpCode::Add => self.exec_add(i, pc),
                OpCode::Sub => self.exec_sub(i, pc),
                OpCode::Mul => self.exec_mul(i, pc),
                OpCode::Div => self.exec_div(i, pc),
                OpCode::Mod => self.exec_mod(i, pc),
                OpCode::Jmp => self.exec_jmp(i, pc),
                OpCode::Jmz => self.exec_jmz(i, pc),
                OpCode::Jmn => self.exec_jmn(i, pc),
                OpCode::Djn => self.exec_djn(i, pc),
                OpCode::Spl => self.exec_spl(i, pc),
                OpCode::Cmp => self.exec_cmp(i, pc),
                OpCode::Seq => self.exec_seq(i, pc),
                OpCode::Sne => self.exec_sne(i, pc),
                OpCode::Slt => self.exec_slt(i, pc),
                OpCode::Ldp => self.exec_ldp(i, pc),
                OpCode::Stp => self.exec_stp(i, pc),
                OpCode::Nop => self.exec_nop(pc),
            }?;

            // requeue process queue if there are still threads
            // TODO: process results of exec_* fns

            // TODO: PostIncrement phase

            Ok(exec_event)
        } else {
            // tried stepping after the core has terminated
            Err(CoreError::AlreadyTerminated)
        }
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

    // TODO: this should just take in a `Field` instead of an `Offset` and
    // `Mode`
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

            AddressingMode::AIndirect
                | AddressingMode::AIndirectPreDecrement
                | AddressingMode::AIndirectPostIncrement => {
                let indirect_offset = self.fetch(direct_addr).a.offset;    

                self.calc_addr(direct_addr, indirect_offset)
            },
            AddressingMode::BIndirect 
                | AddressingMode::BIndirectPreDecrement
                | AddressingMode::BIndirectPostIncrement =>
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
        -> CoreResult<CoreEvent>
    {
        // we can unwrap this because my the time any exec functions have been
        // called, a pid has been loaded into the `last_pid` field
        Ok(CoreEvent::Terminated(self.last_pid.unwrap()))
    }

    /// Execute `mov` instruction
    /// 
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            // A -> A
            OpMode::A  => target.a = source.a,
            // B -> B
            OpMode::B  => target.b = source.b,
            // A -> B
            OpMode::AB => target.b = source.a,
            // B -> A
            OpMode::BA => target.a = source.b,
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
            OpMode::I  => *target = source,
        };

        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `add` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            OpMode::A  => target.a.offset += source.a.offset,
            OpMode::B  => target.b.offset += source.b.offset,
            OpMode::AB => target.b.offset += source.a.offset,
            OpMode::BA => target.a.offset += source.b.offset,
            OpMode::X  => {
                target.a.offset += source.b.offset;
                target.b.offset += source.a.offset;
            }
            OpMode::F
                | OpMode::I => {
                target.a.offset += source.a.offset;
                target.b.offset += source.b.offset;
            }
        }

        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            OpMode::A  => target.a.offset += source.a.offset,
            OpMode::B  => target.b.offset += source.b.offset,
            OpMode::AB => target.b.offset += source.a.offset,
            OpMode::BA => target.a.offset += source.b.offset,
            OpMode::X  => {
                target.a.offset += source.b.offset;
                target.b.offset += source.a.offset;
            }
            OpMode::F
                | OpMode::I => {
                target.a.offset += source.a.offset;
                target.b.offset += source.b.offset;
            }
        }
        
        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `mul` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            OpMode::A  => target.a.offset *= source.a.offset,
            OpMode::B  => target.b.offset *= source.b.offset,
            OpMode::AB => target.b.offset *= source.a.offset,
            OpMode::BA => target.a.offset *= source.b.offset,
            OpMode::X  => {
                target.a.offset *= source.b.offset;
                target.b.offset *= source.a.offset;
            }
            OpMode::F
                | OpMode::I => {
                target.a.offset *= source.a.offset;
                target.b.offset *= source.b.offset;
            }
        }
        
        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `div` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            OpMode::A  => target.a.offset /= source.a.offset,
            OpMode::B  => target.b.offset /= source.b.offset,
            OpMode::AB => target.b.offset /= source.a.offset,
            OpMode::BA => target.a.offset /= source.b.offset,
            OpMode::X  => {
                target.a.offset /= source.b.offset;
                target.b.offset /= source.a.offset;
            }
            OpMode::F
                | OpMode::I => {
                target.a.offset /= source.a.offset;
                target.b.offset /= source.b.offset;
            }
        }
        
        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `mod` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch_mut(target_addr);

        match i.op.mode {
            OpMode::A  => target.a.offset %= source.a.offset,
            OpMode::B  => target.b.offset %= source.b.offset,
            OpMode::AB => target.b.offset %= source.a.offset,
            OpMode::BA => target.a.offset %= source.b.offset,
            OpMode::X  => {
                target.a.offset %= source.b.offset;
                target.b.offset %= source.a.offset;
            }
            OpMode::F
                | OpMode::I => {
                target.a.offset %= source.a.offset;
                target.b.offset %= source.b.offset;
            }
        }
        
        Ok(CoreEvent::Stepped(pc + 1))
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmp(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        Ok(CoreEvent::Jumped(target_addr))
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let target_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target = self.fetch(target_addr);

        let should_jump = match i.op.mode {
            OpMode::A 
                | OpMode::BA => target.a.offset == 0,
            OpMode::B 
                | OpMode::AB => target.b.offset == 0,
            OpMode::X 
                | OpMode::F
                | OpMode::I => target.a.offset == 0 && target.b.offset == 0
        };

        if should_jump {
            Ok(CoreEvent::Jumped(target_addr)) 
        } else {
            Ok(CoreEvent::Stepped(pc + 1))
        }
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        if self.process_count() < self.max_processes {
            let target_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);

            Ok(CoreEvent::Split(pc + 1, target_addr)) // TODO: placeholder
        } else {
            Ok(CoreEvent::Stepped(pc + 1))
        }
    }

    /// Execute `cmp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_cmp(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        self.exec_seq(i, pc)
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch(target_addr);
        
        // FIXME: this is definitively wrong, but I don't know the defined
        // behavior
        let should_jump = match i.op.mode {
            OpMode::A  => source.a.offset == target.a.offset,
            OpMode::B  => source.b.offset == target.b.offset,
            OpMode::AB => source.a.offset == target.b.offset,
            OpMode::BA => source.b.offset == target.a.offset,
            OpMode::X 
                | OpMode::F
                | OpMode::I  => source.a.offset == target.a.offset && 
                                source.b.offset == target.b.offset
        };

        if should_jump {
            Ok(CoreEvent::Jumped(pc + 2))
        } else {
            Ok(CoreEvent::Stepped(pc + 1))
        }
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch(target_addr);
        
        // FIXME: this is definitively wrong, but I don't know the defined
        // behavior
        let should_jump = match i.op.mode {
            OpMode::A  => source.a.offset != target.a.offset,
            OpMode::B  => source.b.offset != target.b.offset,
            OpMode::AB => source.a.offset != target.b.offset,
            OpMode::BA => source.b.offset != target.a.offset,
            OpMode::X 
                | OpMode::F
                | OpMode::I  => source.a.offset != target.a.offset &&
                                source.b.offset != target.b.offset,
        };

        if should_jump {
            Ok(CoreEvent::Stepped(pc + 1))
        } else {
            Ok(CoreEvent::Stepped(pc + 1))
        }
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(&self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        let source_addr = self.calc_target_addr(pc, i.a.offset, i.a.mode);
        let target_addr = self.calc_target_addr(pc, i.b.offset, i.b.mode);

        let source = self.fetch(source_addr);
        let target = self.fetch(target_addr);
        
        // FIXME: this is definitively wrong, but I don't know the defined
        // behavior
        let should_jump = match i.op.mode {
            OpMode::A  => source.a.offset < target.a.offset,
            OpMode::B  => source.b.offset < target.b.offset,
            OpMode::AB => source.a.offset < target.b.offset,
            OpMode::BA => source.b.offset < target.a.offset,
            OpMode::X 
                | OpMode::F
                | OpMode::I  => source.a.offset < target.a.offset &&
                                source.b.offset < target.b.offset,
        };

        if should_jump {
            Ok(CoreEvent::Jumped(pc + 1))
        } else {
            Ok(CoreEvent::Stepped(pc + 2))
        }
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(&mut self, i: Instruction, pc: Address)
        -> CoreResult<CoreEvent>
    {
        unimplemented!();
    }

    /// Execute `nop` instruction
    ///
    /// Supported OpModes: None
    fn exec_nop(&mut self, pc: Address) 
        -> CoreResult<CoreEvent>
    {
        Ok(CoreEvent::Stepped(pc + 1))
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

