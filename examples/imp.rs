/// A core running the imp program

extern crate libcw;
use libcw::redcode::{Instruction, OpCode, OpMode, AddressingMode};
use libcw::simulation::{Core, CoreBuilder, Event};


fn main()
{
    let imp = vec![
        Instruction{ 
            op:     OpCode::Mov,
            mode:   OpMode::I,
            a:      0,
            a_mode: AddressingMode::Direct,
            b:      1,
            b_mode: AddressingMode::Direct
        },
    ];

    let mut core = CoreBuilder::new()
        .core_size(8)
        .load(vec![(0, imp)]).unwrap();

    for i in 0..1 {
        let sim_result = core.step();

        println!("START STEP {}", i);
        println!("{:?}", core);
        println!("END STEP {}", i);

        if sim_result == Ok(Event::Finished) {
            break;
        }
    }
}
