use {nom::*, redcode::*, std::str::FromStr};

named!(
  pub parse_program<&str, Vec<Instruction>>,
  do_parse!(
    instructions: separated_list!(line_ending, parse_instruction)
    >> opt!(line_ending) // trailing whitespace
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
    map!(tag_no_case!("DAT"), |_| OpCode::Dat)
      | map!(tag_no_case!("MOV"), |_| OpCode::Mov)
      | map!(tag_no_case!("ADD"), |_| OpCode::Add)
      | map!(tag_no_case!("SUB"), |_| OpCode::Sub)
      | map!(tag_no_case!("MUL"), |_| OpCode::Mul)
      | map!(tag_no_case!("DIV"), |_| OpCode::Div)
      | map!(tag_no_case!("MOD"), |_| OpCode::Mod)
      | map!(tag_no_case!("JMP"), |_| OpCode::Jmp)
      | map!(tag_no_case!("JMZ"), |_| OpCode::Jmz)
      | map!(tag_no_case!("JMN"), |_| OpCode::Jmn)
      | map!(tag_no_case!("DJN"), |_| OpCode::Djn)
      | map!(tag_no_case!("SPL"), |_| OpCode::Spl)
      | map!(tag_no_case!("CMP"), |_| OpCode::Cmp)
      | map!(tag_no_case!("SEQ"), |_| OpCode::Seq)
      | map!(tag_no_case!("SNE"), |_| OpCode::Sne)
      | map!(tag_no_case!("SLT"), |_| OpCode::Slt)
      | map!(tag_no_case!("LDP"), |_| OpCode::Ldp)
      | map!(tag_no_case!("STP"), |_| OpCode::Stp)
      | map!(tag_no_case!("NOP"), |_| OpCode::Nop)
  )
);

named!(
  parse_addressing_mode<&str, AddressingMode>,
  alt_complete!(
    map!(char!('#'), |_| AddressingMode::Immediate)
      | map!(char!('$'), |_| AddressingMode::Direct)
      | map!(char!('*'), |_| AddressingMode::AIndirect(
        IncrementMode::None
      ))
      | map!(char!('@'), |_| AddressingMode::BIndirect(
        IncrementMode::None
      ))
      | map!(char!('{'), |_| AddressingMode::AIndirect(
        IncrementMode::PreDecrement
      ))
      | map!(char!('<'), |_| AddressingMode::AIndirect(
        IncrementMode::PostIncrement
      ))
      | map!(char!('}'), |_| AddressingMode::BIndirect(
        IncrementMode::PreDecrement
      ))
      | map!(char!('>'), |_| AddressingMode::BIndirect(
        IncrementMode::PostIncrement
      ))
  )
);

named!(
  parse_opmode<&str, OpMode>,
  alt_complete!(
    map!(tag_no_case!("AB"), |_| OpMode::AB)
      | map!(tag_no_case!("BA"), |_| OpMode::BA)
      | map!(tag_no_case!("A"), |_| OpMode::A)
      | map!(tag_no_case!("B"), |_| OpMode::B)
      | map!(tag_no_case!("F"), |_| OpMode::F)
      | map!(tag_no_case!("I"), |_| OpMode::I)
      | map!(tag_no_case!("X"), |_| OpMode::X)
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
          mode: AddressingMode::Immediate,
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
          mode: AddressingMode::Direct,
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
          code: OpCode::Add,
          mode: OpMode::AB,
        },
      ),
      (
        "Mov ",
        OpField {
          code: OpCode::Mov,
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
      ("dAt", OpCode::Dat),
      ("mOv", OpCode::Mov),
      ("aDd", OpCode::Add),
      ("sUb", OpCode::Sub),
      ("mUl", OpCode::Mul),
      ("dIv", OpCode::Div),
      ("mOd", OpCode::Mod),
      ("jMp", OpCode::Jmp),
      ("jMz", OpCode::Jmz),
      ("jMn", OpCode::Jmn),
      ("dJn", OpCode::Djn),
      ("sPl", OpCode::Spl),
      ("cMp", OpCode::Cmp),
      ("sEq", OpCode::Seq),
      ("sNe", OpCode::Sne),
      ("sLt", OpCode::Slt),
      ("lDp", OpCode::Ldp),
      ("sTp", OpCode::Stp),
      ("nOp", OpCode::Nop),
    ];
    for (s, code) in values.iter() {
      assert_eq!(parse_opcode(s).unwrap().1, *code);
    }
  }

  #[test]
  fn test_parse_opmode() {
    let values = [
      ("a ", OpMode::A),
      ("b ", OpMode::B),
      ("Ab ", OpMode::AB),
      ("bA ", OpMode::BA),
      ("x ", OpMode::X),
      ("i ", OpMode::I),
      ("f ", OpMode::F),
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
          code: OpCode::Add,
          mode: OpMode::AB,
        },
        a: Field {
          value: 1,
          mode: AddressingMode::Immediate,
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
        Instruction {
          op: OpField {
            code: OpCode::Add,
            mode: OpMode::AB,
          },
          a: Field {
            value: 4,
            mode: AddressingMode::Immediate
          },
          b: Field {
            value: 3,
            mode: AddressingMode::default(),
          },
        },
        Instruction {
          op: OpField {
            code: OpCode::Mov,
            mode: OpMode::I,
          },
          a: Field {
            value: 2,
            mode: AddressingMode::default()
          },
          b: Field {
            value: 2,
            mode: AddressingMode::AIndirect(IncrementMode::None),
          },
        },
        Instruction {
          op: OpField {
            code: OpCode::Jmp,
            mode: OpMode::default(),
          },
          a: Field {
            value: 2,
            mode: AddressingMode::default()
          },
          b: Field::default()
        },
        Instruction {
          op: OpField {
            code: OpCode::Dat,
            mode: OpMode::default(),
          },
          a: Field {
            value: 0,
            mode: AddressingMode::Immediate
          },
          b: Field {
            value: 0,
            mode: AddressingMode::Immediate,
          },
        },
      ]
    )
  }
}
