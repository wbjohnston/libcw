use {
  nom::*,
  redcode::{AddressingMode::*, OpCode::*, OpMode::*, *},
  std::str::FromStr,
};

named!(
  pub parse_program<&str, Vec<Instruction>>,
  do_parse!(
    instructions: separated_list!(line_ending, parse_instruction)
    >> (instructions)
  )
);

named!(
  parse_instruction<&str, Instruction>,
  do_parse!(
    op: parse_opfield
      >> space0
      >> a: parse_field
      >> space0
      // use the default if it can't be parsed
      >> b: map!(maybe_parse_b_field, |r| r.unwrap_or_default())
      >> (Instruction { op, a, b })
  )
);

named!(
  parse_opcode<&str, OpCode>,
  alt_complete!(
    map!(tag_no_case!("DAT"), |_| Dat)
      | map!(tag_no_case!("MOV"), |_| Mov)
      | map!(tag_no_case!("ADD"), |_| Add)
      | map!(tag_no_case!("SUB"), |_| Sub)
      | map!(tag_no_case!("MUL"), |_| Mul)
      | map!(tag_no_case!("DIV"), |_| Div)
      | map!(tag_no_case!("MOD"), |_| Mod)
      | map!(tag_no_case!("JMP"), |_| Jmp)
      | map!(tag_no_case!("JMZ"), |_| Jmz)
      | map!(tag_no_case!("JMN"), |_| Jmn)
      | map!(tag_no_case!("DJN"), |_| Djn)
      | map!(tag_no_case!("SPL"), |_| Spl)
      | map!(tag_no_case!("CMP"), |_| Cmp)
      | map!(tag_no_case!("SEQ"), |_| Seq)
      | map!(tag_no_case!("SNE"), |_| Sne)
      | map!(tag_no_case!("SLT"), |_| Slt)
      | map!(tag_no_case!("LDP"), |_| Ldp)
      | map!(tag_no_case!("STP"), |_| Stp)
      | map!(tag_no_case!("NOP"), |_| Nop)
  )
);

named!(
  parse_addressing_mode<&str, AddressingMode>,
  alt_complete!(
    map!(char!('#'), |_| Immediate)
      | map!(char!('$'), |_| Direct)
      | map!(char!('*'), |_| AIndirect(
        IncrementMode::None
      ))
      | map!(char!('@'), |_| BIndirect(
        IncrementMode::None
      ))
      | map!(char!('{'), |_| AIndirect(
        IncrementMode::PreDecrement
      ))
      | map!(char!('}'), |_| AIndirect(
        IncrementMode::PostIncrement
      ))
      | map!(char!('<'), |_| BIndirect(
        IncrementMode::PreDecrement
      ))
      | map!(char!('>'), |_| BIndirect(
        IncrementMode::PostIncrement
      ))
  )
);

named!(
  parse_opmode<&str, OpMode>,
  alt_complete!(
    map!(tag_no_case!("AB"), |_| AB)
      | map!(tag_no_case!("BA"), |_| BA)
      | map!(tag_no_case!("A"), |_| A)
      | map!(tag_no_case!("B"), |_| B)
      | map!(tag_no_case!("F"), |_| F)
      | map!(tag_no_case!("I"), |_| I)
      | map!(tag_no_case!("X"), |_| X)
  )
);

named!(
  maybe_parse_b_field<&str, Option<Field>>,
  opt!(
    do_parse!(
      tag!(",")
        >> space0
        >> field: parse_field
        >> (field)
    )
  )
);

named!(
  parse_opfield<&str, OpField>,
  do_parse!(
    code: parse_opcode
      >> mode: map!(maybe_parse_opfield_opmode, |r| r.unwrap_or_default())
      >> (OpField { code, mode })
  )
);

named!(
  maybe_parse_opfield_opmode<&str, Option<OpMode>>,
  opt!(preceded!(char!('.'), parse_opmode))
);

named!(
  parse_field<&str, Field>,
  do_parse!(
    mode: map!(opt!(parse_addressing_mode), |r| r.unwrap_or_default())
    >> space0
    >> value: parse_field_value
    >> (Field { mode, value })
  )
);

named!(
  parse_field_value<&str, Address>,
  map!(digit, |s| FromStr::from_str(s).expect("fasdfasd"))
);

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_parse_field_value() {
    let cases = [("123 ", 123), ("0 ", 0)];
    for (input, expected) in cases.iter() {
      assert_eq!(parse_field_value(input).unwrap().1, *expected);
    }
  }

  #[test]
  fn test_parse_a_field() {
    let cases = [
      (
        "#1 ",
        Field {
          mode: Immediate,
          value: 1,
        },
      ),
      (
        "2 ",
        Field {
          mode: AddressingMode::default(),
          value: 2,
        },
      ),
    ];

    for (input, expected) in cases.iter() {
      assert_eq!(parse_field(input).unwrap().1, *expected);
    }
  }

  #[test]
  fn test_parse_b_field() {
    let cases = [
      // (" EOF", None), // FIXME: why isn't EOF detected
      (
        ", $222 ",
        Some(Field {
          mode: Direct,
          value: 222,
        }),
      ),
    ];
    for (input, expected) in cases.iter() {
      assert_eq!(maybe_parse_b_field(input).unwrap().1, *expected);
    }
  }

  #[test]
  fn test_parse_opfield() {
    let cases = [
      (
        "AdD.Ab ",
        OpField {
          code: Add,
          mode: AB,
        },
      ),
      (
        "Mov ",
        OpField {
          code: Mov,
          mode: OpMode::default(),
        },
      ),
    ];
    for (input, expected) in cases.iter() {
      assert_eq!(parse_opfield(input).unwrap().1, *expected);
    }
  }

  #[test]
  fn test_parse_opcode() {
    let values = [
      ("dAt", Dat),
      ("mOv", Mov),
      ("aDd", Add),
      ("sUb", Sub),
      ("mUl", Mul),
      ("dIv", Div),
      ("mOd", Mod),
      ("jMp", Jmp),
      ("jMz", Jmz),
      ("jMn", Jmn),
      ("dJn", Djn),
      ("sPl", Spl),
      ("cMp", Cmp),
      ("sEq", Seq),
      ("sNe", Sne),
      ("sLt", Slt),
      ("lDp", Ldp),
      ("sTp", Stp),
      ("nOp", Nop),
    ];
    for (s, code) in values.iter() {
      assert_eq!(parse_opcode(s).unwrap().1, *code);
    }
  }

  #[test]
  fn test_parse_opmode() {
    let values = [
      ("a ", A),
      ("b ", B),
      ("Ab ", AB),
      ("bA ", BA),
      ("x ", X),
      ("i ", I),
      ("f ", F),
    ];
    for (s, mode) in values.iter() {
      assert_eq!(parse_opmode(s).unwrap().1, *mode);
    }
  }

  #[test]
  fn test_parse_instruction() {
    let cases = [(
      "AdD.AB #1, $1 ",
      Instruction {
        op: OpField {
          code: Add,
          mode: AB,
        },
        a: Field {
          value: 1,
          mode: Immediate,
        },
        b: Field {
          value: 1,
          mode: AddressingMode::default(),
        },
      },
    )];

    for (s, mode) in cases.iter() {
      assert_eq!(parse_instruction(s).unwrap().1, *mode);
    }
  }

  #[test]
  fn test_parse_program() {
    let program = r#"ADD.AB #4, 3
MOV.I  2, @2
JMP    2
DAT    #0, #0
"#;

    let parsed = parse_program(program).unwrap().1;
    assert_eq!(
      parsed,
      vec![
        Instruction::new(Add, AB, Immediate, 4, AddressingMode::default(), 3),
        Instruction::new(
          Mov,
          I,
          AddressingMode::default(),
          2,
          BIndirect(IncrementMode::None),
          2
        ),
        Instruction::new(
          Jmp,
          OpMode::default(),
          AddressingMode::default(),
          2,
          AddressingMode::default(),
          Address::default()
        ),
        Instruction::new(Dat, OpMode::default(), Immediate, 0, Immediate, 0),
      ]
    )
  }
}
