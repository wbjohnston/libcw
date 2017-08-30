//! Run time error that can occur during simulation

use std::error;
use std::fmt;

/// Kinds of `Simulator` runtime errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error
{
    /// Thrown when trying to step after the simulation has already terminated
    AlreadyTerminated
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "TODO") // TODO
    }
}

impl error::Error for Error
{
    fn description(&self) -> &str
    {
        match *self {
            Error::AlreadyTerminated => 
                "Cannot step after simulator has terminated"
        }
    }
}

