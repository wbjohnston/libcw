extern crate libcw;

const MEMORY_VIEW_SIZE: usize = 17;
const LOAD_OFFSET: usize = 250;

use {
  libcw::{parse_program, Address, Mars},
  std::{
    env, fs,
    io::{self, BufRead, Read, Write},
  },
};

fn main() -> io::Result<()> {
  let stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut input_buffer = String::new();
  let programs = env::args()
    .skip(1) // skip filename
    .filter_map(|path| fs::File::open(path).ok())
    .map(|mut file: fs::File| {
      let mut s = String::new();
      file.read_to_string(&mut s).expect("failed to read file");
      s
    })
    .map(|st| parse_program(st.as_str()).expect("failed to parse").1);

  let mut mars = Mars::default();
  for (i, program) in programs.enumerate() {
    mars.load_program(program.as_slice(), (i * LOAD_OFFSET) as Address);
  }

  while mars.process_count() > 1 {
    let pid = mars.pid().expect("no process running");
    let pc = mars.pc().expect("no process running");

    println!("|PC: {:04} | PID: {:04} |", pc, pid);
    // print  out memory
    for (addr, instr) in mars
      .memory()
      .iter()
      .enumerate()
      .cycle()
      .skip(mars.memory().len() + pc as usize - (MEMORY_VIEW_SIZE - 1) / 2)
      .take(MEMORY_VIEW_SIZE)
    {
      if pc == addr as Address {
        println!(">[{:04}] {}", addr, instr);
      } else {
        println!(" [{:04}] {}", addr, instr);
      }
    }

    print!("> ");
    stdout.flush().expect("failed to flush stdout");
    stdin.lock().read_line(&mut input_buffer)?;

    mars.step();
  }

  println!("last pid, {}", mars.pid().unwrap());

  Ok(())
}
