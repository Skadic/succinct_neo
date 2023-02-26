use crate::int_vec::{num_required_blocks, Fixed, IntVec, IntVector, Dynamic};

impl<const WIDTH: usize> IntVec<Fixed<WIDTH>> {
    /// Creates an integer vector with a given bit width and a default capacity of 8.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Fixed, IntVec};
    ///
    /// let v = IntVec::<Fixed<15>>::new();
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
    /// use succinct_neo::int_vec::{Fixed, IntVec};
    ///
    /// let v = IntVec::<Fixed<15>>::with_capacity(20);
    ///
    /// // 20 integers of size 15 require 300 bits this in turn requiring 5 * 64 blocks (= 320 bits).
    /// // However, 320 bits can hold 21 integers of size 15
    /// assert_eq!(21, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let num_blocks = num_required_blocks::<usize>(capacity, WIDTH);

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width: WIDTH,
            capacity: num_blocks * Self::block_width() / WIDTH,
            size: 0,
            _marker: Default::default()
        };

        temp.data.push(0);
        temp
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * WIDTH) % Self::block_width()
    }

    /// Gets the number of bits each integer is saved with.
    /// In our case, this is the same as the generic type parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Fixed, IntVec};
    ///
    /// let v = IntVec::<Fixed<15>>::new();
    ///
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub const fn bit_width(&self) -> usize {
        WIDTH
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << WIDTH) - 1
    }

    /// Adds an integer to the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `v` - The value to insert.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVec, IntVector, Fixed};
    ///
    /// let mut v = IntVec::<Fixed<10>>::new();
    /// v.push(25);
    /// v.push(8);
    /// v.push(60);
    ///
    /// assert_eq!(25, v.get(0));
    /// assert_eq!(8, v.get(1));
    /// assert_eq!(60, v.get(2));
    /// ```
    pub fn push(&mut self, v: usize) {
        assert!(v < (1 << WIDTH), "value too large for {}-bit integer", WIDTH);
        let offset = self.current_offset();
        let mask = self.mask();
        if offset == 0 {
            *self.data.last_mut().unwrap() |= v & mask;
            self.size += 1;
            return;
        }

        // If we're wrapping into the next block
        if offset + WIDTH >= Self::block_width() {
            let fitting_bits = Self::block_width() - offset;
            let fitting_mask = (1 << fitting_bits) - 1;
            *self.data.last_mut().unwrap() |= (v & fitting_mask) << offset;
            let hi = (v & mask) >> fitting_bits;
            self.data.push(hi);
            self.recalculate_capacity();
            self.size += 1;
            return;
        }

        *self.data.last_mut().unwrap() |= (v & mask) << offset;
        self.size += 1;
    }

    pub fn into_dynamic(self) -> IntVec<Dynamic> {
        // SAFETY: This is in essence the same type with the same memory layout
        unsafe { std::mem::transmute::<Self, IntVec<Dynamic>>(self) }
    }
}

impl<const WIDTH: usize> IntVector for IntVec<Fixed<WIDTH>> {
    unsafe fn get_unchecked(&self, index: usize) -> usize {
        let index_block = (index * WIDTH) / Self::block_width();
        let index_offset = (index * WIDTH) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + WIDTH >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            let remaining_bits = WIDTH - fitting_bits;
            let lo = self.data[index_block] >> index_offset;
            let mask = (1 << remaining_bits) - 1;
            let hi = self.data[index_block + 1] & mask;
            return (hi << fitting_bits) | lo;
        }

        let mask = self.mask();
        (self.data[index_block] >> index_offset) & mask
    }

    fn get(&self, index: usize) -> usize {
        assert!(index < self.len(), "length is {} but index is {index}", self.len());
        unsafe { self.get_unchecked(index) }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: usize) {
        let mask = self.mask();
        let value = value & mask;
        let index_block = (index * WIDTH) / Self::block_width();
        let index_offset = (index * WIDTH) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + WIDTH >= Self::block_width() {
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

        self.data[index_block] &= !(mask << index_offset);
        self.data[index_block] |= value << index_offset;
    }

    fn set(&mut self, index: usize, value: usize) {
        assert!(index < self.len(), "length is {} but index is {index}", self.len());
        assert!(value < (1 << WIDTH), "value {value} too large for {WIDTH}-bit integer");
        unsafe { self.set_unchecked(index, value) }
    }

    fn len(&self) -> usize {
        self.size
    }
}

impl<const WIDTH: usize> Default for IntVec<Fixed<WIDTH>> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::int_vec::{Fixed, IntVec, IntVector};

    #[test]
    fn basics_test() {
        let mut v = IntVec::<Fixed<4>>::new();
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
        let mut v = IntVec::<Fixed<23>>::new();
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
        let mut v = IntVec::<Fixed<7>>::new();
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
        let mut v = IntVec::<Fixed<7>>::new();
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
        let mut v = IntVec::<Fixed<8>>::new();

        for i in 0..20 {
            v.push(i)
        }

        let mut iter = v.iter();
        assert_eq!(20, iter.len(), "incorrect iterator length");
        for (expect, actual) in (0..).zip(&mut iter) {
            assert_eq!(expect, actual, "value at index {expect} incorrect")
        }

        assert_eq!(None, iter.next());
    }

    #[test]
    fn into_iter_test() {
        let mut v = IntVec::<Fixed<12>>::new();
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
        let v = IntVec::<Fixed<7>>::new();
        v.get(10);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut v = IntVec::<Fixed<7>>::new();
        v.set(10, 10);
    }

    #[test]
    #[should_panic]
    fn set_too_large_number_test() {
        let mut v = IntVec::<Fixed<7>>::new();
        v.push(0);
        v.set(0, 100000000);
    }

    #[test]
    #[should_panic]
    fn push_too_large_number_test() {
        let mut v = IntVec::<Fixed<7>>::new();
        v.push(100000000);
    }

    #[test]
    fn shrink_to_fit_test() {
        let mut v = IntVec::<Fixed<9>>::with_capacity(200);

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