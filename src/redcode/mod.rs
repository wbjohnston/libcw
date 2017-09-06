//! Datastructures for representing redcode instructions

/// Alias for a program, which is just a list of instructions
pub type Program = Vec<Instruction>;

/// Address in a core
pub type Address = u32;

/// Offset to an `Address`
pub type Offset = i16;

/// Process ID
pub type Pid = u16;

/// P-space PIN
pub type Pin = Pid;

mod op_code;
pub use self::op_code::OpCode;

mod op_mode;
pub use self::op_mode::OpMode;

mod op_field;
pub use self::op_field::OpField;

mod field;
pub use self::field::Field;

mod addressing_mode;
pub use self::addressing_mode::AddressingMode;

mod instruction;
pub use self::instruction::Instruction;


