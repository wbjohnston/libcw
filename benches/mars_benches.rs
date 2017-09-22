#![feature(test)]

extern crate test;
use test::Bencher;
use test::black_box;

extern crate libcw;
use libcw::redcode::Instruction;
use libcw::redcode::types::*;
use libcw::simulation::{
    Mars,
    MarsBuilder
};

/// Benchmark time it takes to build a `Mars`
#[bench]
fn mars_build_time(bench: &mut Bencher)
{
    bench.iter(|| {
        let _mars: Mars<Instruction> = MarsBuilder::new().build();
    });
}

/// Benchmark time it takes to build and load a program into a `Mars`
#[bench]
fn mars_build_and_load_time(bench: &mut Bencher)
{
    let prog = black_box(vec![
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
        )
    ]);

    let load = vec![(0, None, &prog)];

    bench.iter(|| {
        let _mars = MarsBuilder::new()
            .build_and_load(load.clone());
    });
}

/// Benchmark time to complete a full execution of an imp on default settings
#[bench]
fn mars_imp_sim_time(bench: &mut Bencher)
{
    let prog = black_box(vec![
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
        )
    ]);

    let load = black_box(vec![(0, None, &prog)]);

    let mars = MarsBuilder::new()
        .build_and_load(load)
        .unwrap();

    bench.iter(|| {
        let mut inner_mars = mars.clone();

        while !inner_mars.halted() {
            let _ = inner_mars.step();
        }
    });

}

/// Benchmark the amount of time it takes to simulate a dwarf
#[bench]
fn mars_dwarf_sim_time(bench: &mut Bencher)
{
    let dwarf = vec![
        Instruction::new(
            OpCode::Add,
            Modifier::AB,
            4,
            AddressingMode::Immediate,
            3,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            2,
            AddressingMode::Direct,
            2,
            AddressingMode::BIndirect
            ),
        Instruction::new(
            OpCode::Jmp,
            Modifier::I,
            -2,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Dat,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),
    ]; 

    let load = black_box(vec![(0, None, &dwarf)]);

    let mars = MarsBuilder::new()
        .build_and_load(load)
        .unwrap();

    bench.iter(|| {
        let mut inner_mars = mars.clone();

        while !inner_mars.halted() {
            let _ = inner_mars.step();
        }
    });
}

/// Benchmark the amount of time it takes to simulate a dwarf and imp
/// on the same core
#[bench]
fn mars_imp_vs_dwarf_sim_time(bench: &mut Bencher)
{
    let imp = black_box(vec![
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
        )
    ]);

    let dwarf = vec![
        Instruction::new(
            OpCode::Add,
            Modifier::AB,
            4,
            AddressingMode::Immediate,
            3,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Mov,
            Modifier::I,
            2,
            AddressingMode::Direct,
            2,
            AddressingMode::BIndirect
            ),
        Instruction::new(
            OpCode::Jmp,
            Modifier::I,
            -2,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),
        Instruction::new(
            OpCode::Dat,
            Modifier::I,
            0,
            AddressingMode::Direct,
            0,
            AddressingMode::Direct
            ),
    ]; 

    let load = black_box(vec![(2000, None, &dwarf), (4000, None, &imp)]);

    let mars = MarsBuilder::new()
        .build_and_load(load)
        .unwrap();

    bench.iter(|| {
        let mut inner_mars = mars.clone();

        while !inner_mars.halted() {
            let _ = inner_mars.step();
        }
    });
}

