//! Simulation runtime (aka `Core`) and tools to build a core

// TODO: add functions for hot-loading proceses
// TODO: implement `Core::exec_stp` and `Core::exec_ldp` functions
// TODO: implement p-space PIN's

use std::collections::{VecDeque, HashMap};
use std::num::Wrapping;

use redcode::*;

pub type CoreResult<T> = Result<T, ()>;

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoreEvent
{
    /// All processes terminated successfully
    Finished,

    /// Game ended in a tie
    Tied,

    /// Process split inner contains address of new pc
    Split(Offset),

    /// A process terminated
    Terminated(Pid),

    /// A process jumped address
    Jumped(Offset),

    /// Skipped happens in all `Skip if ...` instructions
    Skipped,

    /// Nothing happened
    Stepped,
}

/// Core wars runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core
{
    /// Core memory
    pub(super) memory:        Vec<Instruction>,

    /// Current process id being run
    pub(super) current_pid:   Pid,

    /// Current program counter
    pub(super) pc:            Address,

    /// Current process queue
    pub(super) current_queue: VecDeque<Address>,

    /// Program counter for each process currently loaded into memory
    pub(super) process_queue: VecDeque<(Pid, VecDeque<Address>)>,

    /// Private storage space for warriors
    pub(super) pspace:        HashMap<Pin, Vec<Instruction>>,

    /// Has the core finished executing 
    pub(super) finished:    bool,

    // Runtime constraints
    /// Core version
    pub(super) version:       usize,

    /// Maximum of processes that can be on the process queue at any time
    pub(super) max_processes: usize,

    /// Maximum number of cycles that can pass before a tie is declared
    pub(super) max_cycles:    usize,
}

impl Core
{
    /// Step forward one cycle
    pub fn step(&mut self) -> CoreResult<CoreEvent>
    {
        // Fetch instruction
        let i = self.fetch(self.pc);
        let (a_mode, b_mode) = (i.a.mode, i.b.mode);

        // preincrement phase
        {
            if a_mode == AddressingMode::AIndirectPreDecrement ||
                a_mode == AddressingMode::BIndirectPreDecrement {
                unimplemented!();
            }

            if b_mode == AddressingMode::AIndirectPreDecrement ||
                b_mode == AddressingMode::BIndirectPreDecrement {
                unimplemented!();
            }
        }

        // Execute instruction
        let exec_event = self.execute(i)?;

        let mut should_requeue = true;
        // Check if core execution has finished
        match exec_event {
            CoreEvent::Stepped => self.current_queue.push_front(self.pc + 1),
            CoreEvent::Terminated(pid) => {
                if self.current_queue.len() == 0 {
                    should_requeue = false; 
                }

                if !should_requeue && self.pids().len() == 0 {
                    self.finished = true;
                    return Ok(CoreEvent::Finished);
                }
            },
            _ => unimplemented!()
        }

        // TODO: postincrement phase
        {
            if a_mode == AddressingMode::AIndirectPostIncrement ||
                a_mode == AddressingMode::BIndirectPostIncrement {
                unimplemented!();
            }

            if b_mode == AddressingMode::AIndirectPostIncrement ||
                b_mode == AddressingMode::BIndirectPostIncrement {
                unimplemented!();
            }
        }
        
        if should_requeue {
            // Requeue local queue onto process queue
            self.process_queue.push_front(
                // FIXME: clone inneficient
                (self.current_pid, self.current_queue.clone()) 
                );
        }

        // Fetch new queue
        let (pid, q) = self.process_queue.pop_back().unwrap();
        self.current_queue = q;
        
        // Update pid and program counter
        self.pc = self.current_queue.pop_back().unwrap();
        self.current_pid = pid;

        Ok(exec_event)
    }

    pub fn execute(&mut self, instr: Instruction) -> CoreResult<CoreEvent>
    {
        if self.finished { // can't execute after finished
            return Err(());
        }

        let (code, mode) = (instr.op.code, instr.op.mode);
        // TODO: should we just do address resolution within the exec_* fns?
        let (a_addr, b_addr) = (
            self.effective_addr_a(),
            self.effective_addr_b()
            );
        
        let exec_event = match code {
            OpCode::Dat => self.exec_dat(),
            OpCode::Mov => self.exec_mov(mode, a_addr, b_addr),
            OpCode::Add => self.exec_add(mode, a_addr, b_addr),
            OpCode::Sub => self.exec_sub(mode, a_addr, b_addr),
            OpCode::Mul => self.exec_mul(mode, a_addr, b_addr),
            OpCode::Div => self.exec_div(mode, a_addr, b_addr),
            OpCode::Mod => self.exec_mod(mode, a_addr, b_addr),
            // OpCode::Mov => 
            _ => unimplemented!()

        };

        Ok(exec_event)
    }

    pub fn finished(&mut self) -> bool
    {
        self.process_queue.is_empty()
    }

    /// Get `Pid` currently executing on the core
    pub fn pc(&self) -> Address
    {
        self.pc
    }

    /// Get the program counters for all processes
    pub fn pcs(&self) -> Vec<Address>
    {
        unimplemented!();
    }
    
    /// Get the current `Pid` executing
    pub fn pid(&self) -> Pid
    {
        self.current_pid
    }

    /// Get all `Pid`s that are currently active
    pub fn pids(&self) -> Vec<Pid>
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
    pub fn memory(&self) -> &[Instruction]
    {
        &self.memory.as_slice()
    }

    /// Get the number of processes currently running
    pub fn process_count(&self) -> usize
    {
        // count length of all local process queues in the global pqueue
        self.process_queue.iter().fold(0, |acc, &(_, ref x)| acc + x.len())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Address resolution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Calculate the address after adding an offset
    fn calc_addr_offset(&self, base: Address, offset: Offset) -> Address
    {
        if offset < 0 {
            (base.wrapping_sub(-offset as Address))
        } else {
            (base.wrapping_add(offset as Address))
        }
    }

    /// Get the effective of address of the current `Instruction`
    fn effective_addr(&self, use_a_field: bool) -> Address
    {
        use self::AddressingMode::*;

        let pc = self.pc;
        let (mode, offset) = {
            let i = self.fetch(self.pc);
            let field = if use_a_field { i.a } else { i.b };

            (field.mode, field.offset)
        };

        match mode {
            Immediate => pc,
            Direct => self.calc_addr_offset(pc, offset),
            AIndirect
                | AIndirectPreDecrement
                | AIndirectPostIncrement => {
                let direct_offset = {
                    let direct = self.fetch(
                        self.calc_addr_offset(self.pc, offset)
                    );
                    direct.a.offset
                };

                self.calc_addr_offset(pc, direct_offset + offset)
            }
            BIndirect
                | BIndirectPreDecrement
                | BIndirectPostIncrement => {
                let direct_offset = {
                    let direct = self.fetch(
                        self.calc_addr_offset(self.pc, offset)
                        );
                    direct.b.offset
                };
                self.calc_addr_offset(pc, direct_offset + offset)
            }
        }
    }

    fn effective_addr_b(&self) -> Address
    {
        self.effective_addr(false)
    }

    /// Get the effective of address of the current `Instruction`'s A Field
    fn effective_addr_a(&self) -> Address
    {
        self.effective_addr(true)
    }

    ////////////////////////////////////////////////////////////////////////////
    // Storage and retrieval functions
    ////////////////////////////////////////////////////////////////////////////

    /// Store an `Instruction` in memory
    fn store(&mut self, addr: Address, instr: Instruction)
    {
        let mem_size = self.size();
        self.memory[addr as usize % mem_size] = instr;
    }

    /// TODO
    fn store_effective_a(&mut self, instr: Instruction)
    {
        let eff_addr = self.effective_addr_a();
        self.store(eff_addr, instr)
    }

    /// TODO
    fn store_effective_b(&mut self, instr: Instruction)
    {
        let eff_addr = self.effective_addr_b();
        self.store(eff_addr, instr)
    }

    /// Fetch copy of instruction in memory
    fn fetch(&self, addr: Address) -> Instruction
    {
        self.memory[addr as usize % self.size()]
    }

    /// TODO
    fn fetch_effective_a(&self) -> Instruction
    {
        self.fetch(self.effective_addr_a())
    }

    /// TODO
    fn fetch_effective_b(&self) -> Instruction
    {
        self.fetch(self.effective_addr_b())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Instruction execution functions
    ////////////////////////////////////////////////////////////////////////////

    /// Execute `dat` instruction
    /// 
    /// Supported OpModes: None
    fn exec_dat(&self) -> CoreEvent
    {
        CoreEvent::Terminated(self.pid())
    }

    /// Execute `mov` instruction
    /// 
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_mov(&mut self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a = a.a,
            OpMode::B => b.b = a.b,
            OpMode::AB =>b.a = a.b,
            OpMode::BA =>b.b = a.a,
            OpMode::F => {
                b.a = a.a;
                b.b = a.b;
            },
            OpMode::X => {
                b.a = a.b;
                b.b = a.a;
            },
            OpMode::I => b = a
        }

        self.store(b_addr, b);
        CoreEvent::Stepped
    }

    /// Execute `add` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_add(&mut self, mode: OpMode, a_addr: Address, b_addr: Address)
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a.offset += a.a.offset,
            OpMode::B => b.b.offset += a.b.offset,
            OpMode::AB =>b.a.offset += a.b.offset,
            OpMode::BA =>b.b.offset += a.a.offset,
            OpMode::F 
                | OpMode::I => {
                b.a.offset += a.a.offset;
                b.b.offset += a.b.offset;
            },
            OpMode::X => {
                b.b.offset += a.a.offset;
                b.a.offset += a.b.offset;
            },
        }

        self.store_effective_b(b);
        CoreEvent::Stepped
    }

    /// Execute `sub` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_sub(&mut self, mode: OpMode, a_addr: Address, b_addr: Address)
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a.offset -= a.a.offset,
            OpMode::B => b.b.offset -= a.b.offset,
            OpMode::AB =>b.a.offset -= a.b.offset,
            OpMode::BA =>b.b.offset -= a.a.offset,
            OpMode::F 
                | OpMode::I => {
                b.a.offset -= a.a.offset;
                b.b.offset -= a.b.offset;
            },
            OpMode::X => {
                b.b.offset -= a.a.offset;
                b.a.offset -= a.b.offset;
            },
        }

        self.store_effective_b(b);
        CoreEvent::Stepped
    }

    /// Execute `mul` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mul(&mut self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a.offset *= a.a.offset,
            OpMode::B => b.b.offset *= a.b.offset,
            OpMode::AB =>b.a.offset *= a.b.offset,
            OpMode::BA =>b.b.offset *= a.a.offset,
            OpMode::F 
                | OpMode::I => {
                b.a.offset *= a.a.offset;
                b.b.offset *= a.b.offset;
            },
            OpMode::X => {
                b.b.offset *= a.a.offset;
                b.a.offset *= a.b.offset;
            },
        }

        self.store_effective_b(b);
        CoreEvent::Stepped
    }

    /// Execute `div` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_div(&mut self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a.offset /= a.a.offset,
            OpMode::B => b.b.offset /= a.b.offset,
            OpMode::AB =>b.a.offset /= a.b.offset,
            OpMode::BA =>b.b.offset /= a.a.offset,
            OpMode::F 
                | OpMode::I => {
                b.a.offset /= a.a.offset;
                b.b.offset /= a.b.offset;
            },
            OpMode::X => {
                b.b.offset /= a.a.offset;
                b.a.offset /= a.b.offset;
            },
        }

        self.store_effective_b(b);
        CoreEvent::Stepped
    }

    /// Execute `mod` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F`
    fn exec_mod(&mut self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        let a     = self.fetch(a_addr);
        let mut b = self.fetch(b_addr);

        match mode {
            OpMode::A => b.a.offset %= a.a.offset,
            OpMode::B => b.b.offset %= a.b.offset,
            OpMode::AB =>b.a.offset %= a.b.offset,
            OpMode::BA =>b.b.offset %= a.a.offset,
            OpMode::F 
                | OpMode::I => {
                b.a.offset %= a.a.offset;
                b.b.offset %= a.b.offset;
            },
            OpMode::X => {
                b.b.offset %= a.a.offset;
                b.a.offset %= a.b.offset;
            },
        }

        self.store_effective_b(b);
        CoreEvent::Stepped
    }

    /// Execute `jmp` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmp(&self, mode: OpMode, a_addr: Address) 
        -> CoreEvent
    {
        let a = self.fetch(a_addr);
        CoreEvent::Jumped(a.a.offset)
    }

    /// Execute `jmz` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmz(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_jmn(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_djn(&self, mode: OpMode, a: Instruction)
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    ///
    /// Supported OpModes: `B`
    fn exec_spl(&self, mode: OpMode, a_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_cmp(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_seq(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_sne(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_slt(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_ldp(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    ///
    /// Supported OpModes: `A` `B` `AB` `BA` `X` `F` `I`
    fn exec_stp(&self, mode: OpMode, a_addr: Address, b_addr: Address) 
        -> CoreEvent
    {
        unimplemented!();
    }

    /// Execute 'nop' instruction
    fn exec_nop(&self) -> CoreEvent
    {
        CoreEvent::Stepped
    }
}

