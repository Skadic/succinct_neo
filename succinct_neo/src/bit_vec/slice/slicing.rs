use std::ops::{Bound, RangeBounds};

use super::BitSlice;

impl<Backing> BitSlice<Backing> {
    /// Gets an immutable view into the data structure without checking for bounds and validity.
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
    /// use succinct_neo::bit_vec::BitVec;
    ///
    /// let bv = BitVec::new(16);
    ///
    /// // extracts bits 4 to inclusively 7.
    /// let slice = unsafe { bv.slice_unchecked(4..8) };
    ///
    /// assert_eq!(4, slice.len());
    /// ```
    ///
    /// # Safety
    ///
    /// The end bound may not be greater than the start bound and both bounds must be at most equal to this slice's length.
    pub unsafe fn slice_unchecked(&self, r: impl RangeBounds<usize>) -> BitSlice<&Backing> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.len(),
        };

        BitSlice::new(&self.backing, self.start + start, self.start + end)
    }

    /// Gets an immutable view into the data structure.
    ///
    /// # Arguments
    ///
    /// * `r`: The range of bits to extract.
    ///
    /// returns: An immutable bit-view into the underlying data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::bit_vec::BitVec;
    ///
    /// let bv = BitVec::new(16);
    ///
    /// // extracts bits 4 to inclusively 7.
    /// let slice = bv.slice(4..8);
    ///
    /// assert_eq!(4, slice.len());
    /// ```
    pub fn slice(&self, r: impl RangeBounds<usize>) -> BitSlice<&Backing> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.len(),
        };

        if start > self.len() {
            panic!("left bound is {start} but length is {}", self.len())
        }
        if end > self.len() {
            panic!("right bound is {end} but length is {}", self.len())
        }
        if start > end {
            panic!("left bound greater than right bound ({start} > {end}) is {end}")
        }

        BitSlice::new(&self.backing, self.start + start, self.start + end)
    }

    /// Gets a mutable view into the data structure without checking for bounds.
    ///
    /// # Arguments
    ///
    /// * `r`: The range of bits to extract.
    ///
    /// returns: A mutable bit-view into the underlying data structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::bit_vec::{BitVec, BitModify, BitGet};
    ///
    /// let mut bv = BitVec::new(16);
    ///
    /// // Extracts bits 4 to inclusively 7.
    /// let mut slice = unsafe { bv.slice_unchecked_mut(4..8) };
    /// assert_eq!(4, slice.len());
    ///
    /// assert_eq!(false, slice.get_bit(3));
    ///
    /// // We can modify the slice!
    /// slice.set_bit(3, true);
    ///
    /// assert_eq!(true, slice.get_bit(3));
    /// ```
    ///
    /// # Safety
    ///
    /// The end bound may not be greater than the start bound and both bounds must be at most equal to this slice's length.
    pub unsafe fn slice_unchecked_mut(
        &mut self,
        r: impl RangeBounds<usize>,
    ) -> BitSlice<&mut Backing> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.len(),
        };

        BitSlice::new(&mut self.backing, self.start + start, self.start + end)
    }

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
    /// use succinct_neo::bit_vec::{BitVec, BitModify, BitGet};
    ///
    /// let mut bv = BitVec::new(16);
    ///
    /// // Extracts bits 4 to inclusively 7.
    /// let mut slice = bv.slice_mut(4..8);
    /// assert_eq!(4, slice.len());
    ///
    /// assert_eq!(false, slice.get_bit(3));
    ///
    /// // We can modify the slice!
    /// slice.set_bit(3, true);
    ///
    /// assert_eq!(true, slice.get_bit(3));
    /// ```
    pub fn slice_mut(&mut self, r: impl RangeBounds<usize>) -> BitSlice<&mut Backing> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.len(),
        };

        if start > self.len() {
            panic!("left bound is {start} but length is {}", self.len())
        }
        if end > self.len() {
            panic!("right bound is {end} but length is {}", self.len())
        }
        if start > end {
            panic!("left bound greater than right bound ({start} > {end}) is {end}")
        }

        BitSlice::new(&mut self.backing, self.start + start, self.start + end)
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Bound, RangeBounds};

    use crate::bit_vec::BitVec;
    /// Range with exclusive start and end index
    struct ExclusiveRange<const S: usize, const E: usize>;

    impl<const S: usize, const E: usize> RangeBounds<usize> for ExclusiveRange<S, E> {
        fn start_bound(&self) -> std::ops::Bound<&usize> {
            Bound::Excluded(&S)
        }

        fn end_bound(&self) -> std::ops::Bound<&usize> {
            Bound::Excluded(&E)
        }
    }

    #[test]
    #[should_panic]
    fn left_out_of_bounds() {
        let bv = BitVec::new(4);
        bv.slice(5..);
    }

    #[test]
    #[should_panic]
    fn right_out_of_bounds() {
        let bv = BitVec::new(4);
        bv.slice(..5);
    }

    #[test]
    #[should_panic]
    fn invalid_bounds() {
        let bv = BitVec::new(4);
        bv.slice(ExclusiveRange::<2, 0>);
    }

    #[test]
    #[should_panic]
    fn left_out_of_bounds_mut() {
        let mut bv = BitVec::new(4);
        bv.slice_mut(5..);
    }

    #[test]
    #[should_panic]
    fn right_out_of_bounds_mut() {
        let mut bv = BitVec::new(4);
        bv.slice_mut(..5);
    }

    #[test]
    #[should_panic]
    fn invalid_bounds_mut() {
        let mut bv = BitVec::new(4);
        bv.slice_mut(ExclusiveRange::<2, 0>);
    }

    #[test]
    fn slice_unchecked_test() {
        let mut bv = BitVec::new(60);

        let mut slice = unsafe { bv.slice_unchecked_mut(20..40) };

        for i in 0..slice.len() {
            slice.set(i, (i / 3) % 2 == 0);
        }

        unsafe {
            assert_eq!(bv.slice(..=30), bv.slice_unchecked(..=30));
            assert_eq!(bv.slice(20..), bv.slice_unchecked(20..));
            assert_eq!(
                bv.slice(ExclusiveRange::<10, 25>),
                bv.slice_unchecked(ExclusiveRange::<10, 25>)
            );

            let bv2 = bv.clone();

            assert_eq!(bv2.slice(..=30), bv.slice_unchecked_mut(..=30));
            assert_eq!(bv2.slice(20..), bv.slice_unchecked_mut(20..));
            assert_eq!(
                bv2.slice(ExclusiveRange::<10, 25>),
                bv.slice_unchecked_mut(ExclusiveRange::<10, 25>)
            );
        }
    }
}
