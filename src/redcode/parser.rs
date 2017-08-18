
// TODO
use super::isa::Instruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token<'a>
{
    text: &'a str,
    kind: TokenKind
}

/// Kind of lexed token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind
{
    /// Operation code
    OpCode,

    /// Opcode modifier
    OpCodeMod,
}

/// State of lexer
#[derive(Debug, PartialEq, Eq)]
struct Lexer<'a>
{
    input: &'a str,
    output: &'a mut Vec<Token<'a>>,
}

impl<'a> Lexer<'a>
{
    /// Completely lex input
    pub fn complete(&mut self) -> Result<(), ()>
    {
        unimplemented!();
    }

    /// Take a character from input
    #[inline]
    fn take(&mut self) -> Result<(), ()>
    {
        unimplemented!();
    }
}

/// Redcode parser
#[derive(Debug, PartialEq, Eq)]
struct Parser<'a>
{
    input: &'a [Token<'a>],
    output: &'a mut Vec<Instruction>,
    stack: Vec<Token<'a>>
}

impl<'a> Parser<'a>
{
    /// Take a Token from input
    #[inline]
    fn take(&mut self) -> Result<(), ()>
    {
        unimplemented!();
    }

    /// Completely parse input
    pub fn complete(&mut self) -> Result<(), ()>
    {
        unimplemented!();
    }
}

/// Parse input string and return struct-ized program
pub fn parse(prog_str: &str) -> Vec<Instruction>
{
    let mut v = vec![];
    parse_into(prog_str, &mut v);
    v
}

/// Parse input string into specified buffer
pub fn parse_into(prog_str: &str, buf: &mut Vec<Instruction>)
{
    unimplemented!();
}

