use crate::int_vec::Iter;
use crate::int_vec::{num_required_blocks, IntVector};
use std::fmt::Debug;

pub struct FixedIntVec<const INT_WIDTH: usize> {
    data: Vec<usize>,
    capacity: usize,
    size: usize,
}

impl<const WIDTH: usize> FixedIntVec<WIDTH> {
    /// Creates an integer vector with a given bit width and a default capacity of 8.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{FixedIntVec, IntVector};
    ///
    /// let v = FixedIntVec::<15>::new();
    ///
    /// // 8 integers of size 15 require 120 bits this in turn requiring 2 * 64 blocks (= 128 bits).
    /// // These can hold 8 integers exactly.
    /// assert_eq!(8, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(8)
    }

    /// Creates an integer vector with a given bit width and capacity.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    /// * `capacity` - The number of integers which should fit into this vector without
    /// reallocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{FixedIntVec, IntVector};
    ///
    /// let v = FixedIntVec::<15>::with_capacity(20);
    ///
    /// // 20 integers of size 15 require 300 bits this in turn requiring 5 * 64 blocks (= 320 bits).
    /// // However, 320 bits can hold 21 integers of size 15
    /// assert_eq!(21, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let num_blocks = num_required_blocks::<usize>(capacity, WIDTH);

        Self {
            data: Vec::with_capacity(num_blocks),
            capacity: num_blocks * Self::block_width() / WIDTH,
            size: 0,
        }
    }

    #[inline]
    const fn block_width() -> usize {
        std::mem::size_of::<usize>() * 8
    }

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

    #[inline]
    fn recalculate_capacity(&mut self) {
        self.capacity = self.data.capacity() * Self::block_width() / WIDTH;
    }

    /// Grants access to the underlying slice where the bits are saved.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVector, FixedIntVec};
    ///
    /// let mut v = FixedIntVec::<32>::new();
    /// v.push(125);
    /// v.push(1231);
    ///
    /// assert_eq!((1231 << 32) | 125, v.raw_data()[0]);
    /// ```
    #[inline]
    pub fn raw_data(&self) -> &[usize] {
        &self.data
    }

    /// Shrinks the allocated backing storage behind this int vector to fit the amount of saved
    /// integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let mut v = DynamicIntVec::with_capacity(5, 200);
    ///
    /// // All these numbers should take 3 bits to save
    /// for i in (0..50) {
    ///     v.push(i % 8)
    /// }
    ///
    /// v.shrink_to_fit();
    ///
    /// // 50 numbers each using 5 bits is 250 bits of storage.
    /// // This fits into 4 * 64 bit blocks (= 256 bits), which in total would fit 51 integers.
    /// assert_eq!(51, v.capacity());
    /// ```
    pub fn shrink_to_fit(&mut self) {
        let required_blocks = num_required_blocks::<usize>(self.size, WIDTH);
        self.data.truncate(required_blocks);
        self.data.shrink_to_fit();
        self.recalculate_capacity();
    }

    /// An iterator over this vector's saved integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{DynamicIntVec, IntVector};
    ///
    /// let mut v = DynamicIntVec::new(10);
    /// for i in (0..20).rev() {
    ///     v.push(i);
    /// }
    ///
    /// assert!(Iterator::eq((0..20).rev(), v.iter()))
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<Self> {
        Iter { i: 0, v: self }
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * WIDTH) % Self::block_width()
    }

    #[inline]
    const fn mask(&self) -> usize {
        usize::MAX >> (Self::block_width() - WIDTH) 
    }

    /// Consumes this int vector and returns the backing [`Vec`].
    pub fn into_inner(self) -> Vec<usize> {
        self.data
    }
}

impl<const WIDTH: usize> IntVector for FixedIntVec<WIDTH> {
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn bit_width(&self) -> usize {
        WIDTH
    }

    unsafe fn get_unchecked(&self, index: usize) -> usize {
        let (index_block, index_offset) = (
            (index * WIDTH) / Self::block_width(),
            (index * WIDTH) % Self::block_width(),
        );

        if !WIDTH.is_power_of_two() {
            // If we're on the border between blocks
            if index_offset + WIDTH > Self::block_width() {
                let fitting_bits = Self::block_width() - index_offset;
                let remaining_bits = WIDTH - fitting_bits;
                let lo = self.data[index_block] >> index_offset;
                let mask = (1 << remaining_bits) - 1;
                let hi = self.data[index_block + 1] & mask;
                return (hi << fitting_bits) | lo;
            }
        }

        let mask = self.mask();
        (self.data[index_block] >> index_offset) & mask
    }

    fn get(&self, index: usize) -> usize {
        assert!(
            index < self.len(),
            "length is {} but index is {index}",
            self.len()
        );
        unsafe { self.get_unchecked(index) }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: usize) {
        let mask = self.mask();
        let value = value & mask;
        let (index_block, index_offset) = (
            (index * WIDTH) / Self::block_width(),
            (index * WIDTH) % Self::block_width(),
        );

        if !WIDTH.is_power_of_two() {
            // If we're on the border between blocks
            if index_offset + WIDTH > Self::block_width() {
                let fitting_bits = Self::block_width() - index_offset;
                unsafe {
                    let lower_block = self.data.get_unchecked_mut(index_block);
                    *lower_block &= !(mask << index_offset);
                    *lower_block |= value << index_offset;
                    let higher_block = self.data.get_unchecked_mut(index_block + 1);
                    *higher_block &= !(mask >> fitting_bits);
                    *higher_block |= value >> fitting_bits;
                }
                return;
            }
        }

        self.data[index_block] &= !(mask << index_offset);
        self.data[index_block] |= value << index_offset;
    }

    fn set(&mut self, index: usize, value: usize) {
        assert!(
            index < self.len(),
            "length is {} but index is {index}",
            self.len()
        );
        debug_assert!(
            value <= self.mask(),
            "value {value} too large for {WIDTH}-bit integer"
        );
        unsafe { self.set_unchecked(index, value) }
    }

    fn push(&mut self, v: usize) {
        debug_assert!(v <= self.mask(), "value too large for {WIDTH}-bit integer");

        let offset = self.current_offset();
        let mask = self.mask();

        if !WIDTH.is_power_of_two() {
            // If we're wrapping into the next block
            if offset + WIDTH > Self::block_width() {
                let fitting_bits = Self::block_width() - offset;
                let fitting_mask = (1 << fitting_bits) - 1;
                *self.data.last_mut().unwrap() |= (v & fitting_mask) << offset;
                let hi = (v & mask) >> fitting_bits;
                self.data.push(hi);
                self.recalculate_capacity();
                self.size += 1;
                return;
            }
        }

        if offset == 0 {
            self.data.push(v & mask);
            self.size += 1;
            return;
        }

        *self.data.last_mut().unwrap() |= (v & mask) << offset;
        self.size += 1;
    }

    fn len(&self) -> usize {
        self.size
    }
}

impl<const WIDTH: usize> Default for FixedIntVec<WIDTH> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize> Debug for FixedIntVec<WIDTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")
            .and_then(|_| {
                let mut iter = self.iter().peekable();
                while let Some(v) = iter.next() {
                    write!(f, "{v}")?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            })
            .and_then(|_| write!(f, "}}"))
    }
}

#[cfg(test)]
mod test {
    use crate::int_vec::{fixed::FixedIntVec, IntVector};

    #[test]
    fn basics_test() {
        let mut v = FixedIntVec::<4>::new();
        assert_eq!(0, v.len(), "int vec size not 0");
        assert!(v.is_empty(), "int vec not empty");

        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);

        assert_eq!(4, v.len(), "int vec size not 4");
        assert!(!v.is_empty(), "int vec not empty");

        assert_eq!(0x4321, v.raw_data()[0], "backing data incorrect");
        println!("{v:?}")
    }

    #[test]
    fn push_test() {
        let mut v = FixedIntVec::<23>::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);

        assert_eq!(1, v.get(0));
        assert_eq!(2, v.get(1));
        assert_eq!(3, v.get(2));
        assert_eq!(4, v.get(3));
    }

    #[test]
    fn set_test() {
        let mut v = FixedIntVec::<7>::new();
        for _ in 0..50 {
            v.push(1);
        }

        for (expected, actual) in std::iter::repeat(1).zip(&v) {
            assert_eq!(expected, actual)
        }

        for (i, val) in (0..50).enumerate() {
            v.set(i, val);
        }

        for (expected, actual) in (0..50).zip(&v) {
            assert_eq!(expected, actual)
        }
    }

    #[test]
    fn get_test() {
        let mut v = FixedIntVec::<7>::new();
        let mut test_v = Vec::new();
        for i in 0..30 {
            v.push(3 * i);
            test_v.push(3 * i);
        }

        for (i, actual) in test_v.into_iter().enumerate() {
            assert_eq!(v.get(i), actual);
        }
    }

    #[test]
    fn iter_test() {
        let mut v = FixedIntVec::<8>::new();

        for i in 0..20 {
            v.push(i)
        }

        let mut iter = v.iter();
        assert_eq!(20, iter.len(), "incorrect iterator length");
        for (expect, actual) in (0usize..).zip(&mut iter) {
            assert_eq!(expect, actual, "value at index {expect} incorrect")
        }

        assert_eq!(None, iter.next());
    }

    #[test]
    fn into_iter_test() {
        let mut v = FixedIntVec::<12>::new();
        let mut test_v = Vec::new();
        let mut i = 1;
        for _ in 0..10 {
            v.push(i);
            test_v.push(i);
            i = (i << 1) | 1;
        }

        let mut iter = v.into_iter();
        assert_eq!(10, iter.len(), "incorrect iterator length");
        for (expect, actual) in test_v.into_iter().zip(&mut iter) {
            assert_eq!(expect, actual);
        }

        assert_eq!(None, iter.next());
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_test() {
        let v = FixedIntVec::<7>::new();
        v.get(10);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut v = FixedIntVec::<7>::new();
        v.set(10, 10);
    }

    #[test]
    #[should_panic]
    fn set_too_large_number_test() {
        let mut v = FixedIntVec::<7>::new();
        v.push(0);
        v.set(0, 100000000);
    }

    #[test]
    #[should_panic]
    fn push_too_large_number_test() {
        let mut v = FixedIntVec::<7>::new();
        v.push(100000000);
    }

    #[test]
    fn shrink_to_fit_test() {
        let mut v = FixedIntVec::<9>::with_capacity(200);

        // 200 * 9 = 1800, which fits into 29 64-bit numbers (= 1856 bits).
        // So the capacity should be 1856 / 9 = 206
        assert_eq!(206, v.capacity, "incorrect capacity before shrink");

        for i in 0..50 {
            v.push(i)
        }

        v.shrink_to_fit();

        // We now have 50 elements in the vector, taking up 50 * 9 = 450 bits and fitting into
        // 8 * 64 bit blocks = 512 bits. These fit 512 / 9 = 56 integers in total.
        assert_eq!(56, v.capacity, "incorrect capacity after shrink");
    }
}
