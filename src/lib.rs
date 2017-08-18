//! Core wars library
//!
//! TODO: longform

use std::collections::VecDeque;

mod isa;
pub use isa::{OpCode, Instruction};

#[derive(Debug, PartialEq, Eq)]
pub enum SimulatorError
{
    NotEnoughMemory,

    PrematureTermination
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimulatorEvent
{
    /// Simulator is comepletely finished executing
    Finished,

    /// Returned if a player executed a dat instruction
    Terminated(usize),
    
    /// Simulator executed and nothing notable happened
    None
}

/// Core wars Simulator
///
/// # Components
/// 1. shared memory: TODO
/// 2. process queue: TODO
#[derive(Debug, PartialEq, Eq)]
pub struct Simulator
{
    /// Simulator memory
    memory:             Vec<Instruction>,

    /// Process count
    pcount:             usize,

    /// Current process id being run
    curr_pid:           usize,

    /// Program counter for each process currently loaded into memory
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
    pub fn step(&mut self) -> Result<SimulatorEvent, SimulatorError>
    {
        
        // decode phase
        // TODO
        
        // fetch phase
        // TODO

        // execution phase
        // TODO

        unimplemented!();
    }

    /// Reset simulator to original state, dumping all currently loaded programs
    pub fn reset(&mut self)
    {
        unimplemented!();
    }

    /// Step the simulator n times
    #[allow(dead_code, unused_variables)]
    pub fn step_n(&mut self, n_steps: usize) 
        -> Result<Vec<SimulatorEvent>, SimulatorError>
    {
        unimplemented!();
    }

    /// Completely simulate 
    pub fn complete(&mut self) -> Result<Vec<usize>, SimulatorError>
    {
        let mut pids = vec![]; // order programs finish in

        loop {
            match self.step()
            {
                Ok(SimulatorEvent::Finished) => break,
                Ok(SimulatorEvent::Terminated(pid)) => pids.push(pid),
                Err(e) => panic!(e),
                _ => { /* do nothing */ },
            };
        }

        Ok(pids)
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

