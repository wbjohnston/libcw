
extern crate libcw;

use std::time;
use std::thread;
use std::cmp;

use libcw::redcode::*;
use libcw::simulation::*;

// Dump the state of the core to stdout
fn print_local_core(core: &Core, margin: usize)
{
    let pc     = core.pc() as usize;
    let m      = core.memory();
    let c_size = core.size();

    let (min, max) = (
        pc.saturating_sub(margin) % c_size,
        cmp::min(pc.saturating_add(margin + 1), core.size())
        );

    // State dump header
    println!("--------------------------------------------------");
    println!(
        "| PC: {} | PID: {} | CYCLE: {} | Viewing {} - {} |",
        pc,
        core.pid(),
        core.cycle(),
        min,
        max
        );
    println!("--------------------------------------------------");

    // Dump memory space
    for i in min..pc {
        println!("| {:^5} | {}", i, m[i % c_size]);
    }

    println!("|>{:^5}<| {}", pc, m[pc % c_size]);

    for i in (pc + 1)..max {
        println!("| {:^5} | {}", i, m[i]);
    }
}

const SLEEP_TIME_MS: u64 = 1000;
const VIEW_MARGIN: usize = 3;

fn main()
{
    let imp = vec![ // Imp program
        Instruction{
            op: OpField {
                code:   OpCode::Mov,
                mode:   OpMode::I
            },
            a: Field {
                offset: 0,
                mode:   AddressingMode::Direct
            },
            b: Field {
                offset: 1,
                mode:   AddressingMode::Direct
            }
        },
    ];

    let dwarf = vec![ // Imp program
        Instruction{
            op: OpField {
                code:   OpCode::Add,
                mode:   OpMode::AB
            },
            a: Field {
                offset: 4,
                mode:   AddressingMode::Immediate
            },
            b: Field {
                offset: 3,
                mode:   AddressingMode::Direct
            }
        },
        Instruction{
            op: OpField {
                code:   OpCode::Mov,
                mode:   OpMode::I
            },
            a: Field {
                offset: 2,
                mode:   AddressingMode::Direct
            },
            b: Field {
                offset: 2,
                mode:   AddressingMode::BIndirect
            }
        },
        Instruction{
            op: OpField {
                code:   OpCode::Jmp,
                mode:   OpMode::I
            },
            a: Field {
                offset: -2,
                mode:   AddressingMode::Direct
            },
            b: Field {
                offset: 0,
                mode:   AddressingMode::Direct
            }
        },
        Instruction{
            op: OpField {
                code:   OpCode::Dat,
                mode:   OpMode::I
            },
            a: Field {
                offset: 0,
                mode:   AddressingMode::Immediate
            },
            b: Field {
                offset: 0,
                mode:   AddressingMode::Immediate
            }
        },
    ];

    let sleep_duration = time::Duration::from_millis(SLEEP_TIME_MS);

    // Create a core with our programs in it
    let mut core = CoreBuilder::new()
        .core_size(128)
        .load(vec![
            (64, None, dwarf.clone()),
            (00, None, imp.clone()),
        ])
        .unwrap();

    // Print intial state
    print_local_core(&core, VIEW_MARGIN);

    'main: loop {
        let event = core.step();
        print_local_core(&core, VIEW_MARGIN);

        thread::sleep(sleep_duration);
        match event {
            Ok(CoreEvent::Finished)
                | Ok(CoreEvent::Tied) =>
            {
                println!("Core Terminated");
                break 'main;
            }
            _ => {}
        }
    }

    println!("Remaining PIDS: {:?}", core.pids());
}
