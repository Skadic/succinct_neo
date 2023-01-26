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
    fn len(&self) -> usize;

    unsafe fn get_unchecked(&self, index: usize) -> usize;
    fn get(&self, index: usize) -> usize {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }
        // SAFETY: We checked that the index is in bounds
        unsafe { self.get_unchecked(index) }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: usize);
    fn set(&mut self, index: usize, value: usize) {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }
        // SAFETY: We checked that the index is in bounds
        unsafe { self.set_unchecked(index, value) }
    }
}

pub trait BitGet {
    unsafe fn get_unchecked(&self, index: usize) -> bool;
    fn get(&self, index: usize) -> bool;
}

pub trait BitModify {
    unsafe fn set_unchecked(&mut self, index: usize, value: bool);
    fn set(&mut self, index: usize, value: bool);
    unsafe fn flip_unchecked(&mut self, index: usize);
    fn flip(&mut self, index: usize);
}

pub trait SliceBit<Index>: Sized {
    fn slice<'a>(&'a self, index: Index) -> BitSlice<'a, Self>;
}

pub trait SliceBitMut<Index>: Sized {
    fn slice_mut<'a>(&'a mut self, index: Index) -> BitSliceMut<'a, Self>;
}

pub trait BitAccess: BitGet + BitModify {}
impl<T> BitAccess for T where T: BitGet + BitModify {}
