//! Your one-stop shop for everything Core Wars

pub mod redcode;

#[cfg(feature = "mars")]
pub mod simulation;


#[cfg(feature = "parser")]
#[macro_use] extern crate lazy_static;

#[cfg(feature = "parser")]
extern crate regex;

#[cfg(feature = "parser")]
pub mod parser;

