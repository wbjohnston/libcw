//! Tools for parsing strings into usable redcode instructions

// TODO: even though I literally just lifted this to a toplevel file, I can 
// move the error information struct and enum into it's own file

use redcode::Program;

/// Result of a parse
pub type ParseResult<T> = Result<T, ParseError>;

/// Holds state for lexing
struct Lexer; // TODO

/// Holds state for parsing
struct Parser; // TODO: implement parser structure

/// Structure containing all data about an error occuring during parsing
pub struct ParseError;

/// Kinds of errors the parser can throw
enum ParseErrorKind {} // TODO

/// Unit of information from an input program
struct Token; // TODO

/// Type of token
enum TokenKind {} // TODO

/// Parse a string into `Instruction`s placing them in a buffer
/// 
/// # Arguments
/// * `program_str`: text of program
/// * `buf`: buffer to place parsed data in
///
/// # Return
/// Vector contained `Instruction`s `program_str` was parsed into
pub fn parse_into(program_str: &str, buf: &mut Program)
    -> ParseResult<()>
{
    unimplemented!();
}

/// Parse a string into `Instruction`s
/// # Arguments
/// `program_str`: text of program
///
/// # Return
/// Vector contained `Instruction`s `program_str` was parsed into
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
fn lex(program_str: &str) 
    -> ParseResult<Vec<Token>>
{
    unimplemented!();
}

/// Parse tokens into a vector of `Instructions`
///
/// # Arguments
/// * `program_str`: text of program
///
/// # Return
/// parsed program on success `ParseError` otherwise
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
fn parse_tokens_into(program_str: Vec<Token>, buf: &mut Program)
    -> ParseResult<()>
{
    unimplemented!();
}

