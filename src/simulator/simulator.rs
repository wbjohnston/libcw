//! Redcode simulator

use std::collections::VecDeque;
use std::collections::HashMap;

use redcode::*;

pub type SimulatorResult = Result<SimulatorEvent, SimulatorError>;

/// Insruction that a core is loaded with by default
pub const DEFAULT_INSTRUCTION: Instruction = Instruction {
    op: OpField { mode: OpMode::I, op: OpCode::Dat },
    a:  Field   { mode: AddressingMode::Direct, offset: 0 },
    b:  Field   { mode: AddressingMode::Direct, offset: 0 },
};

/// Simulator runtime errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulatorError
{
    // Nothing here.
    AllWarriorsTerminated
}

/// Events that can happen during a running simulation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimulatorEvent
{
    /// All processes terminated successfully
    Finished,

    Tied,

    /// A process terminated
    Terminated(usize),

    /// Nothing happened
    None,
}

/// Core wars Simulator
#[derive(Debug)]
pub struct Simulator
{
    /// Simulator memory
    memory:        Vec<Instruction>,

    /// Current process id being run
    active_pid:    usize,

    /// Core version
    version:       usize,

    /// Maximum of processes that can be on the process queue at any time
    max_processes: usize,

    /// Program counter for each process currently loaded into memory
    process_queue: VecDeque<(usize, VecDeque<usize>)>,

    /// Private storage space for warriors
    pspace:        HashMap<usize, Vec<Instruction>>
}

impl Simulator
{
    /// Step forward one cycle
    pub fn step(&mut self) -> SimulatorResult
    {
        // TODO: this is written pretty badly

        // get active process counter
        // TODO: better error handling
        if let Some((pid, mut q)) = self.process_queue.pop_back() {
            self.active_pid = pid;
            let pc = q.pop_back().unwrap(); 

            // fetch phase
            let i = self.memory[pc];

            // match i.a.mode {
            //     AddressingMode::AIndirectPreDecrement => {
            //         self.memory[pc + i.a.offset].a.offset -= 1;
            //     },
            //     AddressingMode::BIndirectPreDecrement => {
            //         self.memory[pc + i.a.offset].b.offset -= 1;
            //     },
            //     _ => {}
            // };

            // match i.b.mode {
            //     AddressingMode::AIndirectPreDecrement => {
            //         self.memory[pc + i.b.offset].a.offset -= 1;
            //     },
            //     AddressingMode::BIndirectPreDecrement => {
            //         self.memory[pc + i.b.offset].b.offset -= 1;
            //     },
            //     _ => {}
            // };

            // execution phase
            let (mode, a, b) = (i.op.mode, i.a, i.b); 
            let exec_event = match i.op.op {
                OpCode::Dat => self.exec_dat(),
                OpCode::Mov => self.exec_mov(mode, a, b),
                OpCode::Add => self.exec_add(mode, a, b),
                OpCode::Sub => self.exec_sub(mode, a, b),
                OpCode::Mul => self.exec_mul(mode, a, b),
                OpCode::Div => self.exec_div(mode, a, b),
                OpCode::Mod => self.exec_mod(mode, a, b),
                OpCode::Jmp => self.exec_jmp(mode, a, b),
                OpCode::Jmz => self.exec_jmz(mode, a, b),
                OpCode::Jmn => self.exec_jmn(mode, a, b),
                OpCode::Djn => self.exec_djn(mode, a, b),
                OpCode::Spl => self.exec_spl(mode, a, b),
                OpCode::Cmp => self.exec_cmp(mode, a, b),
                OpCode::Seq => self.exec_seq(mode, a, b),
                OpCode::Sne => self.exec_sne(mode, a, b),
                OpCode::Slt => self.exec_slt(mode, a, b),
                OpCode::Ldp => self.exec_ldp(mode, a, b),
                OpCode::Stp => self.exec_stp(mode, a, b),
                OpCode::Nop => self.exec_nop(),
            }?;

            // requeue process queue
            self.process_queue.push_front((pid, q));

            // TODO: post increment
            Ok(exec_event)
        } else {
            Ok(SimulatorEvent::Finished)
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
    fn exec_mov(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_add(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_sub(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_mul(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_div(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_mod(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_jmp(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field // FIXME: don't think this is necessary
        )
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
    fn exec_jmz(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_jmn(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_djn(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_spl(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_cmp(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_seq(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_sne(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_slt(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_ldp(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
    fn exec_stp(&mut self, 
        mode: OpMode,
        a: Field,
        b: Field
        )
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
}

