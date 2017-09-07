/// A core running the dwarf program

extern crate libcw;
use libcw::redcode::*;
use libcw::simulation::*;

fn main()
{
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

    let mut core = CoreBuilder::new()
        .core_size(8)
        .max_cycles(8)
        .load(vec![(0, None, dwarf)]).unwrap();

    println!("INITIAL STATE START");
    for (i, instr) in core.memory().iter().enumerate() {
        if i as Address == core.pc() {
            println!("PC> {:?}", instr);
        } else {
            println!("    {:?}", instr);
        }
    }
    println!("INITIAL STATE END");

    let mut events = vec![];

    for i in 0.. {
        events.push(core.step());

        /// print out core
        println!("START STEP {}", core.cycle());
        for (i, instr) in core.memory().iter().enumerate() {
            if i as Address == core.pc() {
                println!("PC> {:?}", instr);
            } else {
                println!("    {:?}", instr);
            }
        }
        println!("END STEP {}", i);

        if core.finished() {
            println!("TERMINATED");
            break;
        }
    }

    println!("Events: {:?}", events);
}
