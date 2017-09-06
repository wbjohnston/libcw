/// A core running the imp program

extern crate libcw;
use libcw::redcode::*;
use libcw::simulation::*;

fn main()
{
    let imp = vec![
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

    let mut core = CoreBuilder::new()
        .max_cycles(10)
        .core_size(8)
        .load(vec![(0, None, imp.clone()), (4, None, imp.clone())])
        .unwrap();

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
