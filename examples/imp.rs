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
        .core_size(8)
        .load(vec![(0, None, imp.clone()), (4, None, imp.clone())])
        .unwrap();

    println!("INITIAL STATE");
    for (i, instr) in core.memory().iter().enumerate() {
        if i as Address == core.pc() {
            println!("PC> {:?}", instr);
        } else {
            println!("    {:?}", instr);
        }
    }
    println!("INITIAL STATE");

    for i in 1..10 {
        let sim_result = core.step();

        println!("START STEP {}", i);
        for (i, instr) in core.memory().iter().enumerate() {
            if i as Address == core.pc() {
                println!("PC> {:?}", instr);
            } else {
                println!("    {:?}", instr);
            }
        }
        println!("END STEP {}", i);

        if sim_result == Ok(CoreEvent::Finished) {
            break;
        }
    }
    println!("TERMINATED");
}
