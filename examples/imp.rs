//! An example of running a core with the common warrior, the imp, loaded.

use std::cmp;

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
    let pc = mars.pc().unwrap() as usize;
    let pid = mars.pid().unwrap();
    let cycle = mars.cycle();
    let size = mars.size();

    // print header
    println!("| Cycle: {} | PC: {} | PID: {} |", cycle, pc, pid);

    let (min, max) = (
            pc.saturating_sub(margin),
            pc.saturating_add(margin) % size
        );

    for i in min..max {
        println!("|{}| {}", i, mars.memory()[i]);
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
                offset: 0,
                mode: AddressingMode::Direct
            },
            b: Field {
                offset: 1,
                mode: AddressingMode::Direct
            }
        }
    ]; 

    // create mars
    let mut mars = MarsBuilder::new()
        .max_cycles(10)
        .build_and_load(vec![(4000, None, &imp)])
        .unwrap();

    // display initial state
    display_mars_state(&mars, 5);

    // run
    while !mars.halted() {
        let event = mars.step(); 
        display_mars_state(&mars, 5);
    }
}

