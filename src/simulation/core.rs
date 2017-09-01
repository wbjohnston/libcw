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
    // TODO: Determine if `exec_*(src: &Instruction, trg: &mut Instruction)`
    // is a bad API
    /// Step forward one cycle
    pub fn step(&mut self)
        -> CoreResult<CoreEvent>
    {
        // FIXME: this is written pretty badly
        // get active process counter
        if let Some((pid, mut q)) = self.process_queue.pop_back() {
            self.last_pid = Some(pid);
            let pc = q.pop_back().unwrap(); 

            let i = self.fetch(pc).clone();
            let src_addr = self.calc_target_addr(pc, i.a);
            let trg_addr = self.calc_target_addr(pc, i.b);
            let (code, mode) = (i.op.code, i.op.mode);

            // FIXME: fix borrow checker, need to use `split_at_mut(usize)`
            // to get two mutable slices of the memory buffer
            let src = &self.fetch(src_addr).clone();
            let trg = self.fetch_mut(trg_addr);

            // execution phase
            let exec_event = match code {
                OpCode::Dat => Ok(CoreEvent::Terminated(pid)),
                OpCode::Mov => {
                    Self::exec_mov(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }, 
                OpCode::Add => {
                    Self::exec_add(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Sub => {
                    Self::exec_sub(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Mul => {
                    Self::exec_mul(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Div => {
                    Self::exec_div(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Mod => {
                    Self::exec_mod(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Jmp => {
                    Self::exec_jmp(mode, src);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Jmz => {
                    Self::exec_jmz(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Jmn => {
                    Self::exec_jmn(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Djn => {
                    Self::exec_djn(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Spl => {
                    Self::exec_spl(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                },
                OpCode::Cmp => {
                    Self::exec_cmp(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Seq => {
                    Self::exec_seq(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                },
                OpCode::Sne => {
                    Self::exec_sne(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                },
                OpCode::Slt => {
                    Self::exec_slt(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Ldp => {
                    Self::exec_ldp(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Stp => {
                    Self::exec_stp(mode, src, trg);
                    Ok(CoreEvent::Stepped(pc + 1))
                }
                OpCode::Nop => Ok(CoreEvent::Stepped(pc + 1)),
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
        let addr = self.calc_addr(addr, 0);
        &mut self.memory[addr as usize]
    }

    /// Calculate an address plus an offset taking core size into account
    /// # Arguments
    /// * `base`: base address
    /// * `offst`: offset of base to calculate
    #[inline]
    fn calc_addr(&self, base: Address, offset: Offset) -> Address
    {
        unimplemented!("Need to handle underflow overflow within core bounds");
    }

    // TODO: this should just take in a `Field` instead of an `Offset` and
    // `Mode`
    /// TODO: docs
    fn calc_target_addr(&self, addr: Address, field: Field) -> Address
    {
        unimplemented!("This probably doesn't work right now");
        // calculate first so we don't have to do multiple function calls
        // let direct_addr = self.calc_addr(addr, field.offset);

        // match field.mode {
        //     AddressingMode::Direct => direct_addr,
        //     AddressingMode::AIndirect
        //         | AddressingMode::AIndirectPreDecrement
        //         | AddressingMode::AIndirectPostIncrement => {
        //         let indirect_offset = self.fetch(direct_addr).a.offset;    

        //         self.calc_addr(direct_addr, indirect_offset)
        //     },
        //     AddressingMode::BIndirect 
        //         | AddressingMode::BIndirectPreDecrement
        //         | AddressingMode::BIndirectPostIncrement =>
        //     {
        //         let indirect_offset = self.fetch(direct_addr).b.offset;    
        //         self.calc_addr(direct_addr, indirect_offset)
        //     },
        //     AddressingMode::Immediate => unreachable!() 
        // }
    }

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
                trg.b      = src.a;
                trg.a      = src.b;
            }
            OpMode::F  => {
                trg.a      = src.a;
                trg.b      = src.b;
            }
            OpMode::I  => *trg = *src,
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
        }
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
        }
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
        }
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
        }
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmp(mode: OpMode, src: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_cmp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(mode: OpMode, src: &Instruction, trg: &Instruction)
    {
        unimplemented!();
    }
    /////////////
    // Data accessors
    /////////////
}

