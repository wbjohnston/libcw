//! An example of running a core with the warriors im and dwarf loaded

use std::thread;
use std::time;
use std::fmt;

extern crate libcw;
use libcw::redcode::types::*;
use libcw::redcode::Instruction;
use libcw::redcode::traits;
use libcw::simulation::{MarsBuilder, Mars};

/// Display the state of the MARS on `stdout`
///
/// # Arguments
/// * `mars`: pointer to `Mars`
/// * `margin`: memory addresses before and after pc to display
fn display_mars_state<T>(mars: &Mars<T>, margin: usize)
    where T: traits::Instruction + fmt::Display
{
    let pc    = mars.pc() as usize;
    let pid   = mars.pid();
    let cycle = mars.cycle();
    let size  = mars.size();

    // print header
    println!("| Cycle: {} | PC: {} | PID: {} |", cycle, pc, pid);

    let min = if margin > pc {
        size - (margin - pc) 
    } else {
        pc - margin
    };

    let iter = mars.memory().iter()
        .enumerate()
        .cycle()
        .skip(min)
        .take(margin*2 + 1);

    for (addr, ins) in iter {
        if addr == pc {
            println!(">{}< {}", addr, ins);
        } else {
            println!("|{}| {}", addr, ins);
        }
    }
}

fn main()
{
    let imp = vec![
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            0,
            AddressingMode::Direct,
            1,
            AddressingMode::Direct
            )
    ]; 

    let dwarf = vec![
        Instruction::new(
            OpCode::Add,
            Modifier::AB,
            4,
            AddressingMode::Immediate,
            3,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            2,
            AddressingMode::Direct,
            2,
            AddressingMode::BIndirect
            ),
        Instruction::new(
            OpCode::Jmp,
            Modifier::I,
            -2,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Dat,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),

    ]; 

    // create mars
    let mut mars = MarsBuilder::new()
        .build_and_load(vec![
            (2000, None, &imp),
            (6000, None, &dwarf),
        ])
        .unwrap();

    // display initial state
    display_mars_state(&mars, 25);

    // run
    while !mars.halted() {
        thread::sleep(time::Duration::from_millis(1000));
        let _ = mars.step(); 
        display_mars_state(&mars, 25);
    }
}

