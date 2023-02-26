use crate::int_vec::{Fixed, IntVec};

impl<const WIDTH: usize> IntVec<Fixed<WIDTH>> {

    /// Gets the number of bits each integer is saved with.
    /// In our case, this is the same as the generic type parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let v = IntVec::new(15);
    ///
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub const fn bit_width(&self) -> usize {
        WIDTH
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << self.bit_width()) - 1
    }
}