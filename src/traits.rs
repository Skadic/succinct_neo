use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use num::{
    traits::{NumAssignOps, NumOps},
    FromPrimitive, Integer, ToPrimitive, Unsigned,
};

pub trait BlockType:
    Unsigned
    + Integer
    + Copy
    + FromPrimitive
    + ToPrimitive
    + NumOps
    + NumAssignOps
    + BitOr
    + BitAnd
    + BitXor
    + BitOrAssign
    + BitAndAssign
    + BitXorAssign
{
}

impl BlockType for usize {}
impl BlockType for u128 {}
impl BlockType for u64 {}
impl BlockType for u32 {}
impl BlockType for u16 {}
impl BlockType for u8 {}
