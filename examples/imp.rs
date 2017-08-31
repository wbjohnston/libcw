/// A core running the imp program

extern crate libcw;
use libcw::redcode::{
    Instruction,
    OpCode,
    OpMode,
    AddressingMode,
    OpField,
    Field
    };
use libcw::simulation::{Core, CoreBuilder, Event};


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
