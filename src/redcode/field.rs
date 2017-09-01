
use std::ops;

use redcode::{Offset, AddressingMode};

/// Field containing addressing mode and offset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Field
{
    pub offset: Offset,
    pub mode:   AddressingMode
}

impl ops::AddAssign for Field 
{
    #[inline]
    fn add_assign(&mut self, other: Field)
    {
        unimplemented!();
    }
}

impl ops::MulAssign for Field
{
    #[inline]
    fn mul_assign(&mut self, other: Field)
    {
        unimplemented!();
    }

}

impl ops::DivAssign for Field
{
    #[inline]
    fn div_assign(&mut self, other: Field)
    {
        unimplemented!();
    }

}

impl ops::SubAssign for Field
{
    #[inline]
    fn sub_assign(&mut self, other: Field)
    {
        unimplemented!();
    }

}

impl ops::RemAssign for Field
{
    #[inline]
    fn rem_assign(&mut self, other: Field)
    {
        unimplemented!();
    }

}

impl Default for Field
{
    fn default() -> Self
    {
        Field {
            offset: 0,
            mode: AddressingMode::Direct
        }
    }
}

