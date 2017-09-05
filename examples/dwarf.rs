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
        .load(vec![(0, dwarf)]).unwrap();

    for i in 0..4 {
        let sim_result = core.step();

        println!("START STEP {}", i);
        for instr in core.memory() {
            println!("{:?}", instr);
        }
        println!("END STEP {}", i);

        if sim_result == Ok(CoreEvent::Finished) {
            break;
        }
    }
}
