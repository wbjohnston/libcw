//! Tools for parsing strings into usable redcode instructions

use regex::Regex;

use redcode::Program;

use std::collections::HashMap;

/// Result of a parse
#[allow(dead_code, unused_variables)]
pub type ParseResult<T> = Result<T, ParseError>;

/// Holds state for lexing
#[allow(dead_code, unused_variables)]
struct Lexer<'a>
{
    input:  &'a str,
    output: &'a mut Vec<Token<'a>>
}

/// Holds state for parsing
#[allow(dead_code, unused_variables)]
struct Parser<'a>
{
    sym_table: HashMap<String, String>,
    input:     &'a Vec<Token<'a>>,
    output:    &'a mut Program
}

/// Structure containing all data about an error occuring during parsing
#[allow(dead_code, unused_variables)]
pub struct ParseError;

/// Kinds of errors the parser can throw
#[allow(dead_code, unused_variables)]
enum ParseErrorKind {} // TODO

/// Unit of information from an input program
#[allow(dead_code, unused_variables)]
struct Token<'a>
{
    content: &'a str,
    start:   usize,
    end:     usize,
    kind:    TokenKind,
}

/// Type of token
#[allow(dead_code, unused_variables)]
enum TokenKind
{
    /// Jump label
    Label,

    /// "MOV", "DAT" ...
    OpCode,

    /// "A", "B", ...
    OpMode,
    
    /// "$", "#" ...
    AddressingMode,
    
    /// "+", "-" ...
    Symbol,

    /// Number Literaly
    Number,
    
    /// String literal
    Identifier,
}

/// Parse a string into `Instruction`s placing them in a buffer
///
/// # Arguments
/// * `program_str`: text of program
/// * `buf`: buffer to place parsed data in
///
/// # Return
/// Vector contained `Instruction`s `program_str` was parsed into
#[allow(dead_code, unused_variables)]
pub fn parse_into(program_str: &str, buf: &mut Program)
    -> ParseResult<()>
{
    let tokens = lex(program_str);
    // TODO: symbol resolution (labels, EQU, ...)
    // TODO: expression resolution
    unimplemented!();
}

/// Parse a string into `Instruction`s
/// # Arguments
/// `program_str`: text of program
///
/// # Return
/// Vector contained `Instruction`s `program_str` was parsed into
#[allow(dead_code, unused_variables)]
pub fn parse(program_str: &str)
    -> ParseResult<Program>
{
    let mut v = vec![];
    parse_into(program_str, &mut v)?;
    Ok(v)
}

/// Convert a string into `Token` vector
///
/// # Aruguments
/// * `program_str`: text of program
///
/// # Return
/// vector containing all tokens on success `ParseError` otherwise
#[allow(dead_code, unused_variables)]
fn lex<'a>(program_str: &'a str)
    -> ParseResult<Vec<Token<'a>>>
{
    let mut buf = vec![];
    lex_into(program_str, &mut buf)?;
    Ok(buf)
}

fn lex_into<'a>(program_str: &'a str, buf: &'a mut Vec<Token>)
    -> ParseResult<()>
{
    unimplemented!();
    Ok(())
}

/// Parse tokens into a vector of `Instructions`
///
/// # Arguments
/// * `program_str`: text of program
///
/// # Return
/// parsed program on success `ParseError` otherwise
#[allow(dead_code, unused_variables)]
fn parse_tokens(program_str: Vec<Token>)
    -> ParseResult<Program>
{
    let mut v = vec![];
    parse_tokens_into(program_str, &mut v)?;
    Ok(v)
}

/// Parse tokens into a vector of `Instructions` placing them in a buffer
///
/// # Arguments
/// * `prog`: program to parse
///
/// # Return
/// `Ok(())` on success and `ParseError` otherwise
#[allow(dead_code, unused_variables)]
fn parse_tokens_into(program_str: Vec<Token>, buf: &mut Program)
    -> ParseResult<()>
{
    unimplemented!();
}

