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
                offset: -1,
                mode:   AddressingMode::Direct
            }
        },
    ];

    let mut core = CoreBuilder::new()
        .core_size(8)
        .load(vec![(0, imp)])
        .unwrap();

    println!("START STEP 0");
    for instr in core.memory() {
        println!("{:?}", instr);
    }
    println!("END STEP 0");

    for i in 1..4 {
        let sim_result = core.step();

        println!("START STEP {}", i);
        for instr in core.memory() {
            println!("{:?}", instr);
        }
        println!("END STEP {}", i);

        if sim_result == Ok(CoreEvent::Finished) ||
            sim_result == Err(CoreError::AlreadyTerminated)
        {
            break;
        }
    }
}
