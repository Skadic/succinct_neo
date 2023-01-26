use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use num::{
    traits::{NumAssignOps, NumOps},
    FromPrimitive, Integer, ToPrimitive, Unsigned,
};

use crate::bit_vec::slice::{BitSlice, BitSliceMut};

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

pub trait IntAccess {
    unsafe fn get_unchecked(&self, index: usize) -> usize;
    fn get(&self, index: usize) -> usize;

    unsafe fn set_unchecked(&mut self, index: usize, value: usize);
    fn set(&mut self, index: usize, value: usize);
}

pub trait BitGet {
    unsafe fn get_unchecked(&self, index: usize) -> bool;
    fn get(&self, index: usize) -> bool;
}

impl<T: BitGet> BitGet for &'_ T {
    #[inline]
    unsafe fn get_unchecked(&self, index: usize) -> bool {
        <T as BitGet>::get_unchecked(self, index)
    }

    #[inline]
    fn get(&self, index: usize) -> bool {
        <T as BitGet>::get(self, index)
    }
}

impl<T: BitGet> BitGet for &'_ mut T {
    #[inline]
    unsafe fn get_unchecked(&self, index: usize) -> bool {
        <T as BitGet>::get_unchecked(self, index)
    }

    #[inline]
    fn get(&self, index: usize) -> bool {
        <T as BitGet>::get(self, index)
    }
}

pub trait BitModify {
    unsafe fn set_unchecked(&mut self, index: usize, value: bool);
    fn set(&mut self, index: usize, value: bool);
    unsafe fn flip_unchecked(&mut self, index: usize);
    fn flip(&mut self, index: usize);
}

pub trait SliceBit<Index>: Sized {
    fn slice(&self, index: Index) -> BitSlice<'_, Self>;
}

pub trait SliceBitMut<Index>: Sized {
    fn slice_mut(&mut self, index: Index) -> BitSliceMut<'_, Self>;
}

pub trait BitAccess: BitGet + BitModify {}
impl<T> BitAccess for T where T: BitGet + BitModify {}
