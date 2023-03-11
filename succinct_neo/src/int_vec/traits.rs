/// Allows access to integers in a datastructure
pub trait IntVector {

    /// Returns the amount of integers would fit into the currently allocated memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let v = DynamicIntVec::with_capacity(5, 50);
    ///
    /// // 50 integers of 5 bit each, would fit into 250 bits in total which would make 4 * 64 bit
    /// // blocks, making 256 bits in total. However, 256 bits fit 51 integers of size 5.
    /// assert_eq!(51, v.capacity());
    /// ```
    fn capacity(&self) -> usize;

    /// Gets the number of bits each integer is saved with.
    /// In our case, this is the same as the generic type parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let v = DynamicIntVec::new(15);
    ///
    /// assert_eq!(15, v.bit_width());
    /// ```
    fn bit_width(&self) -> usize;

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


    /// Adds an integer to the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `v` - The value to insert.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let mut v = DynamicIntVec::new(10);
    /// v.push(25);
    /// v.push(8);
    /// v.push(60);
    ///
    /// assert_eq!(25, v.get(0));
    /// assert_eq!(8, v.get(1));
    /// assert_eq!(60, v.get(2));
    /// ```
    fn push(&mut self, v: usize);

    /// The amount of integers saved in this vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let mut v = DynamicIntVec::new(10);
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
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let mut v = DynamicIntVec::new(32);
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
