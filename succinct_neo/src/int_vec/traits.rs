
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

