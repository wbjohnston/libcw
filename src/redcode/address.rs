
use std::ops;
use std::cmp::{PartialOrd, Ordering};

use redcode::Offset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address 
{
    value: usize,
    max:   usize
}

impl Address
{
    /// Create a new `Address`
    /// 
    /// # Arguments
    /// * `value`: value
    /// * `max`: maximum value the `Address` can take before it wraps
    pub fn new(value: usize, max: usize) -> Self
    {
        Address { value: value % max, max }
    }
}

impl ops::Add<Offset> for Address
{
    type Output = Self;

    #[inline]
    fn add(self, other: Offset) -> Self
    {
        unimplemented!();
    }
}

impl ops::Sub<Offset> for Address
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Offset) -> Self
    {
        unimplemented!();
    }
}

impl ops::Mul<Offset> for Address
{
    type Output = Self;

    #[inline]
    fn mul(self, other: Offset) -> Self
    {
        unimplemented!();
    }
}


impl ops::Div<Offset> for Address
{
    type Output = Self;

    #[inline]
    fn div(self, other: Offset) -> Self
    {
        unimplemented!();
    }
}

impl ops::Rem<Offset> for Address
{
    type Output = Self;

    #[inline]
    fn rem(self, other: Offset) -> Self
    {
        unimplemented!();
    }
}

impl PartialOrd for Address
{
    fn partial_cmp(&self, other: &Address) -> Option<Ordering>
    {
        unimplemented!();
    }
}

