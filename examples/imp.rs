//! An example of running a core with the common warrior, the imp, loaded.

use std::thread;
use std::time;

extern crate libcw;
use libcw::redcode::*;
use libcw::simulation::{MarsBuilder, Mars};

/// Display the state of the MARS on `stdout`
///
/// # Arguments
/// * `mars`: pointer to `Mars`
/// * `margin`: memory addresses before and after pc to display
fn display_mars_state(mars: &Mars, margin: usize)
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
        Instruction {
            op: OpField {
                code: OpCode::Mov,
                mode: OpMode::I
            },
            a: Field {
                value: 0,
                mode: AddressingMode::Direct
            },
            b: Field {
                value: 1,
                mode: AddressingMode::Direct
            }
        }
    ]; 

    // create mars
    let mut mars = MarsBuilder::new()
        .max_cycles(16)
        .size(16)
        .build_and_load(vec![(4000, None, &imp)])
        .unwrap();

    // display initial state
    display_mars_state(&mars, 5);

    // run
    while !mars.halted() {
        thread::sleep(time::Duration::from_millis(1000));
        let _ = mars.step(); 
        display_mars_state(&mars, 5);
    }
}

