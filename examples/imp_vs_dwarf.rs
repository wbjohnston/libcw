
extern crate libcw;

use std::time;
use std::thread;

use libcw::redcode::*;
use libcw::simulation::*;

fn print_local_core(core: &Core, margin: usize)
{
    let pc     = core.pc() as usize;
    let m      = core.memory();
    let c_size = core.size();

    let (min, max) = (
        pc.saturating_sub(margin) % core.size(),
        pc.saturating_add(margin) % core.size()
        );

    println!("| PC: {} | PID: {} | CYCLE: {} |", pc, core.pid(), core.cycle());
    
    // Scroll down
    for i in ((pc + 1)..max).rev() {
        println!(" {} : {}", i, m[i % c_size]);
    }

    println!("[{}]: {}", pc, m[pc % c_size]);

    for i in (min..pc).rev() {
        println!(" {} : {}", i, m[i % c_size]);
    }
}

const SLEEP_TIME_MS: u64 = 1000;
const VIEW_MARGIN: usize = 8;

fn main()
{
    // let imp = vec![
    //     Instruction{ 
    //         op: OpField {
    //             code:   OpCode::Mov,
    //             mode:   OpMode::I
    //         },
    //         a: Field {
    //             offset: 0,
    //             mode:   AddressingMode::Direct
    //         },
    //         b: Field {
    //             offset: 1,
    //             mode:   AddressingMode::Direct
    //         }
    //     },
    // ];
    
    let dwarf = vec![
        Instruction{ 
            op: OpField {
                code:   OpCode::Add,
                mode:   OpMode::I
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
                offset: -1,
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

    let mut core = CoreBuilder::new()
        .core_size(16)
        .load(vec![
              // (0, None, imp.clone()),
              (8, None, dwarf.clone())
        ])
        .unwrap();

    print_local_core(&core, VIEW_MARGIN);

    'main: loop {
        let event = core.step();
        print_local_core(&core, VIEW_MARGIN);
        
        thread::sleep(sleep_duration);
        match event {
            Ok(CoreEvent::Finished) => {
                println!("Core Terminated");
                break 'main;
            }
            _ => {}
        }
    }
}
