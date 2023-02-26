/// Allows access to integers in a datastructure
pub trait IntVector {
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

    /// The amount of integers saved in this vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec, IntVector};
    ///
    /// let mut v = IntVec::<Dynamic>::new(10);
    /// for i in 0..20 {
    ///     v.push(i);
    /// }
    ///
    /// assert_eq!(20, v.len());
    ///
    /// ```
    fn len(&self) -> usize;

    /// Checks whether this has no integers saved.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec, IntVector};
    ///
    /// let mut v = IntVec::<Dynamic>::new(32);
    ///
    /// assert!(v.is_empty());
    ///
    /// v.push(125);
    ///
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
