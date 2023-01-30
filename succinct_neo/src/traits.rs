use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use num::{
    traits::{NumAssignOps, NumOps},
    FromPrimitive, Integer, ToPrimitive, Unsigned,
};

use crate::bit_vec::slice::{BitSlice, BitSliceMut};

/// A trait supporting basic primitve integer operations, used for block types in compressed data structures.
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

/// Allows access to integers in a datastructure
pub trait IntAccess {
    /// Gets an integer without making any checks for bounds etc.
    ///
    /// # Safety
    ///
    /// In general, this expects `index` to be in bounds of the datastructure.
    /// However, other type-specific contracts might exist.
    unsafe fn get_unchecked(&self, index: usize) -> usize;

    /// Gets the integer at an index while checking for bounds.
    fn get(&self, index: usize) -> usize;

    /// Sets an integer to the given value without making any checks for bounds etc.
    ///
    /// # Safety
    ///
    /// In general, this expects `index` to be in bounds of the datastructure and
    /// the value to fit the word width of the data structure.
    /// However, other type-specific contracts might exists.
    unsafe fn set_unchecked(&mut self, index: usize, value: usize);

    /// Sets the integer at an index to the given value while checking for bounds and other requirements.
    fn set(&mut self, index: usize, value: usize);
}

/// Defines methods for accessing bits stored in a datastructure.
pub trait BitGet {
    /// Gets a bit without making any checks for bounds.
    ///
    /// # Safety
    ///
    /// In general, this expects `index` to be in bounds of the datastructure.
    /// However, other type-specific contracts might exist.
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool;

    /// Gets the value at an index while checking for bounds.
    fn get_bit(&self, index: usize) -> bool;
}

impl<T: BitGet> BitGet for &'_ T {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        <T as BitGet>::get_bit_unchecked(self, index)
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        <T as BitGet>::get_bit(self, index)
    }
}

impl<T: BitGet> BitGet for &'_ mut T {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        <T as BitGet>::get_bit_unchecked(self, index)
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        <T as BitGet>::get_bit(self, index)
    }
}

/// Defines methods for modifying bits stored in a datastructure.
pub trait BitModify {
    /// Sets a bit to a boolean value without making any checks for bounds.
    ///
    /// # Safety
    ///
    /// In general, this expects `index` to be in bounds of the datastructure.
    /// However, other type-specific contracts might exist.
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool);

    /// Sets a bit to a boolean value while making any checks for bounds.
    fn set_bit(&mut self, index: usize, value: bool);

    /// Flips a bit without making any checks for bounds.
    ///
    /// # Safety
    ///
    /// In general, this expects `index` to be in bounds of the datastructure.
    /// However, other type-specific contracts might exist.
    unsafe fn flip_bit_unchecked(&mut self, index: usize);

    /// Flips a bit while making any checks for bounds.
    fn flip_bit(&mut self, index: usize);
}

/// Allows retrieving an immutable view into a bit-storing data structure.
pub trait SliceBit<Index>: Sized {
    /// Gets an immutable view into the data structure.
    fn slice_bits(&self, index: Index) -> BitSlice<'_, Self>;
}

/// Allows retrieving a mutable view into a bit-storing data structure.
pub trait SliceBitMut<Index>: Sized {
    /// Gets a mutable view into the data structure.
    fn slice_bits_mut(&mut self, index: Index) -> BitSliceMut<'_, Self>;
}

pub trait BitAccess: BitGet + BitModify {}
impl<T> BitAccess for T where T: BitGet + BitModify {}
