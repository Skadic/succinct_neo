use crate::int_vec::{num_required_blocks, Dynamic, IntVec, IntVector};

impl IntVec<Dynamic> {
    /// Creates an integer vector with a given bit width and a default capacity of 8.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let v = IntVec::<Dynamic>::new(15);
    ///
    /// // 8 integers of size 15 require 120 bits this in turn requiring 2 * 64 blocks (= 128 bits).
    /// // These can hold 8 integers exactly.
    /// assert_eq!(8, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn new(width: usize) -> Self {
        Self::with_capacity(width, 8)
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
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let v = IntVec::<Dynamic>::with_capacity(15, 20);
    ///
    /// // 20 integers of size 15 require 300 bits this in turn requiring 5 * 64 blocks (= 320 bits).
    /// // However, 320 bits can hold 21 integers of size 15
    /// assert_eq!(21, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        let num_blocks = num_required_blocks::<usize>(capacity, width);

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width,
            capacity: num_blocks * Self::block_width() / width,
            size: 0,
            _marker: Default::default(),
        };

        temp.data.push(0);
        temp
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * self.width) % Self::block_width()
    }

    /// Gets the number of bits each integer is saved with.
    /// In our case, this is the same as the generic type parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let v = IntVec::<Dynamic>::new(15);
    ///
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub const fn bit_width(&self) -> usize {
        self.width
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << self.width) - 1
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
    /// use succinct_neo::int_vec::{Dynamic, IntVec, IntVector};
    ///
    /// let mut v = IntVec::<Dynamic>::new(10);
    /// v.push(25);
    /// v.push(8);
    /// v.push(60);
    ///
    /// assert_eq!(25, v.get(0));
    /// assert_eq!(8, v.get(1));
    /// assert_eq!(60, v.get(2));
    /// ```
    pub fn push(&mut self, v: usize) {
        if v >= (1 << self.width) {
            panic!("value too large for {}-bit integer", self.width)
        }
        let offset = self.current_offset();
        let mask = self.mask();
        if offset == 0 {
            *self.data.last_mut().unwrap() |= v & mask;
            self.size += 1;
            return;
        }

        // If we're wrapping into the next block
        if offset + self.width >= Self::block_width() {
            let fitting_bits = Self::block_width() - offset;
            let fitting_mask = (1 << fitting_bits) - 1;
            let mask = (1 << self.width) - 1;
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

    /// Modifies this vector to require the minimum amount of bits per saved element.
    ///
    /// This searches for the largest element in the vector. It then saves all saved all integers
    /// with its number of required bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let mut v = IntVec::<Dynamic>::with_capacity(9, 25);
    ///
    /// // All these numbers should take 3 bits to save
    /// for i in (0..50).step_by(2) {
    ///     v.push(i % 8)
    /// }
    ///
    /// v.bit_compress();
    ///
    /// assert_eq!(3, v.bit_width());
    /// ```
    pub fn bit_compress(&mut self) {
        let Some(min_required_bits) = self.iter().reduce(|acc, v| { acc.max(v) }).map(|min| if min > 1 { (min - 1).ilog2() as usize + 1 } else { 1 }) else {
            // No elements in here
            return;
        };

        debug_assert!(min_required_bits <= self.width, "minimum required bits for the elements in this vector greater than previous word width");

        let old_width = self.width;
        self.width = min_required_bits;
        self.recalculate_capacity();

        for i in 0..self.len() {
            // SAFETY: we know the amount of values in this bitvector, so there's no problem
            unsafe {
                let v = self.get_unchecked_with_width(i, old_width);
                self.set_unchecked_with_width(i, v, self.width)
            }
        }
    }
}

impl IntVector for IntVec<Dynamic> {
    unsafe fn get_unchecked(&self, index: usize) -> usize {
        self.get_unchecked_with_width(index, self.width)
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
        self.set_unchecked_with_width(index, value, self.width)
    }

    fn set(&mut self, index: usize, value: usize) {
        assert!(
            index < self.len(),
            "length is {} but index is {index}",
            self.len()
        );
        assert!(
            value < (1 << self.width),
            "value {value} too large for {}-bit integer",
            self.width
        );
        unsafe { self.set_unchecked(index, value) }
    }

    fn len(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod test {
    use crate::int_vec::{Dynamic, IntVec, IntVector};

    #[test]
    fn basics_test() {
        let mut v = IntVec::<Dynamic>::new(4);
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
        let mut v = IntVec::<Dynamic>::new(23);
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
        let mut v = IntVec::<Dynamic>::new(7);
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
        let mut v = IntVec::<Dynamic>::new(7);
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
        let mut v = IntVec::<Dynamic>::new(8);

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
        let mut v = IntVec::<Dynamic>::new(12);
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
        let v = IntVec::<Dynamic>::new(7);
        v.get(10);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut v = IntVec::<Dynamic>::new(7);
        v.set(10, 10);
    }

    #[test]
    #[should_panic]
    fn set_too_large_number_test() {
        let mut v = IntVec::<Dynamic>::new(7);
        v.push(0);
        v.set(0, 100000000);
    }

    #[test]
    #[should_panic]
    fn push_too_large_number_test() {
        let mut v = IntVec::<Dynamic>::new(7);
        v.push(100000000);
    }

    #[test]
    fn bit_compress_test() {
        let mut v = IntVec::<Dynamic>::with_capacity(9, 25);

        // 25 * 9 = 225, which fits into 4 64-bit numbers (= 256 bits).
        // So the capacity should be 256 / 9 = 28
        assert_eq!(28, v.capacity, "incorrect capacity before compression");

        // All these numbers should take 3 bits to save
        for i in (0..50).step_by(2) {
            v.push(i % 8)
        }

        v.bit_compress();

        assert_eq!(3, v.width, "incorrect word width after compression");

        // We were at 256 bits before with a bit size of 3.
        // So 256 / 3 = 85
        assert_eq!(85, v.capacity, "incorrect capacity after compression");
        assert_eq!(25, v.len(), "incorrect length after compression");

        for i in 0..v.len() {
            assert_eq!((2 * i) % 8, v.get(i), "incorrect value at index {i}")
        }
    }

    #[test]
    fn shrink_to_fit_test() {
        let mut v = IntVec::<Dynamic>::with_capacity(9, 200);

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
