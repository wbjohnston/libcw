
use redcode::*;
use simulator::{SimulatorError, SimulatorEvent};

use std::collections::VecDeque;

pub type SimulatorResult = Result<SimulatorEvent, SimulatorError>;

/// Core wars Simulator
///
/// # Components
/// 1. shared memory: TODO
/// 2. process queue: TODO
#[derive(Debug)]
pub struct Simulator
{
    /// Simulator memory
    memory:             Vec<Instruction>,

    /// Process count
    pcount:             usize,

    /// Current process id being run
    curr_pid:           usize,

    /// Program counter for each process currently loaded into memory
    // TODO: implement as VecDeque<(usize, VecDeque<usize>)>
    // so that I can rely soely on VecDeque primitives
    process_queue:      Vec<VecDeque<usize>>
}

impl Simulator
{
    ////////////
    // Mutators
    ////////////
    /// Load a program into memory and add pid (player id)
    ///
    /// # Arguments
    /// * `program`: program to load into memory
    /// * `offset`: offset in memory the program will be loaded into
    pub fn load(&mut self, program: &Vec<Instruction>, offset: usize)
        -> Result<usize, SimulatorError>
    {
        let msize = self.memory.len();

        if program.len() > msize {
            // program will overwrite itself if its loaded into memory
            Err(SimulatorError::NotEnoughMemory)        
        } else { // copy program into memory
            for i in 0..program.len() {
                // programs wrap
                self.memory[(i + offset) % msize] = program[i];
            }

            // add to process queue
            let mut new_q = VecDeque::new();
            new_q.push_front(offset);
            self.process_queue.push(new_q);

            self.pcount += 1;
            Ok(0)
        }
    }

    /// Step the simulator one instruction
    pub fn step(&mut self) -> SimulatorResult
    {
        // decode phase
        // TODO: actual decode phase
        let pc = 0;
        
        // fetch phase
        let i = self.memory[pc];

        // TODO: predecrement

        // execution phase
        let (mode, a, b) = (i.op.mode, i.a, i.b); 
        let exec_event = match i.op.opcode {
            OpCode::Dat => self.exec_nop(mode, a, b),
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
            OpCode::Nop => self.exec_nop(mode, a, b),
        }?;

        // TOOD: post increment
        Ok(exec_event)
    }


    /// Reset simulator to original state, dumping all currently loaded programs
    pub fn reset(&mut self)
    {
        unimplemented!();
    }

    /// Completely simulate 
    pub fn complete(&mut self) -> Result<Vec<usize>, SimulatorError>
    {
        let mut pids = vec![]; // order programs finish in

        loop {
            match self.step()?
            {
                SimulatorEvent::Finished        => break,
                SimulatorEvent::Terminated(pid) => pids.push(pid),
                SimulatorEvent::None            => {},
            };
        }

        Ok(pids)
    }

    /////////////
    // Instruction Execution functions
    /////////////
    /// Execute `dat` instruction
    fn exec_dat(&mut self,
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `mov` instruction
    fn exec_mov(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `add` instruction
    fn exec_add(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `sub` instruction
    fn exec_sub(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `mul` instruction
    fn exec_mul(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `div` instruction
    fn exec_div(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `mod` instruction
    fn exec_mod(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmp` instruction
    fn exec_jmp(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmz` instruction
    fn exec_jmz(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `jmn` instruction
    fn exec_jmn(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `djn` instruction
    fn exec_djn(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `spl` instruction
    fn exec_spl(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `cmp` instruction
    fn exec_cmp(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `seq` instruction
    fn exec_seq(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `sne` instruction
    fn exec_sne(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `slt` instruction
    fn exec_slt(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `ldp` instruction
    fn exec_ldp(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `stp` instruction
    fn exec_stp(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
    }

    /// Execute `nop` instruction
    fn exec_nop(&mut self, 
                mode: OpMode,
                a: InstructionField,
                b: InstructionField
                )
        -> SimulatorResult
    {
        unimplemented!();
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
    pub fn current_pid(&self) -> usize
    {
        self.curr_pid
    }

    /// The number of programs currently loaded into memory
    #[inline]
    pub fn pcount(&self) -> usize
    {
        self.pcount
    }
}
