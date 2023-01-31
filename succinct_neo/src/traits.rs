use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use num::{
    traits::{NumAssignOps, NumOps},
    FromPrimitive, Integer, ToPrimitive, Unsigned,
};

use crate::bit_vec::slice::BitSlice;

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
    /// Get a bit without checking for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index from which to read the bit.
    ///
    /// returns: `true` if the index is a 1, `false` otherwise.
    ///
    /// # Safety
    ///
    /// Contracts depend on the data structure, but in general, the index must be in bounds.
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool;

    /// Get a bit checking for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index from which to read the bit.
    ///
    /// returns: `true` if the index is a 1, `false` otherwise.
    fn get_bit(&self, index: usize) -> bool;
}

impl<T: BitGet> BitGet for &'_ T {
    #[inline]
    #[allow(clippy::missing_safety_doc)]
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

impl<T: BitModify> BitModify for &'_ mut T {
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit_unchecked(self, index, value)
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit(self, index, value)
    }

    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        <T as BitModify>::flip_bit_unchecked(self, index)
    }

    fn flip_bit(&mut self, index: usize) {
        <T as BitModify>::flip_bit(self, index)
    }
}

/// Defines methods for modifying bits stored in a datastructure.
pub trait BitModify {
    /// Sets a bit to a boolean value while not making any checks for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index at which to set the bit.
    /// * `value`: The value to set the bit to
    ///
    /// # Safety
    ///
    /// Contracts depend on the data structure, but in general, the index must be in bounds.
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool);

    /// Sets a bit to a boolean value while checking for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index at which to set the bit.
    /// * `value`: The value to set the bit to
    fn set_bit(&mut self, index: usize, value: bool);

    /// Flips a bit while not making any checks for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the bit  to flip.
    ///
    /// # Safety
    ///
    /// Contracts depend on the data structure, but in general, the index must be in bounds.
    unsafe fn flip_bit_unchecked(&mut self, index: usize);

    /// Flips a bit while checking for bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the bit to flip.
    fn flip_bit(&mut self, index: usize);
}

/// Allows retrieving a view into a bit-storing data structure.
pub trait SliceBit<Index>: Sized {

    /// Gets an immutable view into the data structure.
    ///
    /// # Arguments
    ///
    /// * `index`: The index which defines the slice to extract.
    ///
    /// returns: An immutable bit-view into the underlying data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     traits::SliceBit,
    /// };
    ///
    /// let bv = BitVec::new(16);
    ///
    /// // extracts bits 4 to inclusively 7.
    /// let slice = bv.slice_bits(4..8);
    ///
    /// assert_eq!(4, slice.len());
    /// ```
    fn slice_bits(&self, index: Index) -> BitSlice<&Self>;

    /// Gets a mutable view into the data structure.
    ///
    /// # Arguments
    ///
    /// * `index`: The index which defines the slice to extract.
    ///
    /// returns: A mutable bit-view into the underlying data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     traits::{SliceBit, BitModify, BitGet},
    /// };
    ///
    /// let mut bv = BitVec::new(16);
    ///
    /// // Extracts bits 4 to inclusively 7.
    /// let mut slice = bv.slice_bits_mut(4..8);
    /// assert_eq!(4, slice.len());
    ///
    /// assert_eq!(false, slice.get_bit(3));
    ///
    /// // We can modify the slice!
    /// slice.set_bit(3, true);
    ///
    /// assert_eq!(true, slice.get_bit(3));
    /// ```
    fn slice_bits_mut(&mut self, index: Index) -> BitSlice<&mut Self>;
}

pub trait BitAccess: BitGet + BitModify {}
impl<T> BitAccess for T where T: BitGet + BitModify {}
