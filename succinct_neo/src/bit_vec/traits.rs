use std::rc::Rc;

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

impl<T: BitGet + ?Sized> BitGet for &'_ T {
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

impl<T: BitGet + ?Sized> BitGet for &'_ mut T {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        <T as BitGet>::get_bit_unchecked(self, index)
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        <T as BitGet>::get_bit(self, index)
    }
}

impl<T: BitModify + ?Sized> BitModify for &'_ mut T {
    #[inline]
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit_unchecked(self, index, value)
    }

    #[inline]
    fn set_bit(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit(self, index, value)
    }

    #[inline]
    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        <T as BitModify>::flip_bit_unchecked(self, index)
    }

    #[inline]
    fn flip_bit(&mut self, index: usize) {
        <T as BitModify>::flip_bit(self, index)
    }
}

impl<T: BitGet + ?Sized> BitGet for Box<T> {
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

impl<T: BitModify + ?Sized> BitModify for Box<T> {
    #[inline]
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit_unchecked(self, index, value)
    }

    #[inline]
    fn set_bit(&mut self, index: usize, value: bool) {
        <T as BitModify>::set_bit(self, index, value)
    }

    #[inline]
    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        <T as BitModify>::flip_bit_unchecked(self, index)
    }

    #[inline]
    fn flip_bit(&mut self, index: usize) {
        <T as BitModify>::flip_bit(self, index)
    }
}

impl<T: BitGet> BitGet for Rc<T> {
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

pub trait BitAccess: BitGet + BitModify {}
impl<T> BitAccess for T where T: BitGet + BitModify {}
