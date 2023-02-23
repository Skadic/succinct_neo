pub use traits::IntAccess;

mod traits;

#[derive(Debug)]
pub struct IntVec {
    data: Vec<usize>,
    width: usize,
    capacity: usize,
    size: usize,
}

impl IntVec {

    /// Gets the number of required blocks of the given type to contain the specified number of
    /// elements of a given width.
    ///
    /// # Arguments
    ///
    /// * `num_elements` - The number of elements intended to be saved.
    /// * `bit_width` - The bit width of each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// // 32 * 10 makes 320 bits, requiring 5 * 64bit blocks. 
    /// assert_eq!(5, IntVec::num_required_blocks::<u64>(32, 10))
    /// ```
    #[inline]
    pub fn num_required_blocks<T>(num_elements: usize, bit_width: usize) -> usize {
        (num_elements as f64 * bit_width as f64 / (std::mem::size_of::<T>() as f64 * 8.0)).ceil() as usize
    }

    #[inline]
    fn recalculate_capacity(&mut self) {
        self.capacity = self.data.capacity() * Self::block_width() / self.width;
    }

    #[inline]
    pub fn new(width: usize) -> Self {
        Self::with_capacity(width, 8)
    }

    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        let num_blocks = Self::num_required_blocks::<usize>(capacity, width);

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width,
            capacity: num_blocks * Self::block_width() / width,
            size: 0,
        };

        temp.data.push(0);
        temp
    }

    #[inline]
    const fn block_width() -> usize {
        std::mem::size_of::<usize>() * 8
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << self.width) - 1
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * self.width) % Self::block_width()
    }

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

    #[inline]
    pub fn raw_data(&self) -> &[usize] {
        &self.data
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter { i: 0, v: self }
    }

    /// Gets an integer of the given bit width from an index.
    ///
    /// This gets the `width` bits at bit index `index width`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to read from.
    /// * `width` - The bit width of integers.
    ///
    /// # Safety
    ///
    /// If `n` is the amount of bits stored in this vector, then for `index` and `width`
    /// `(index + 1) * width < n` must hold.
    ///
    unsafe fn get_unchecked_with_width(&self, index: usize, width: usize) -> usize {
        let index_block = (index * width) / Self::block_width();
        let index_offset = (index * width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + width >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            let remaining_bits = width - fitting_bits;
            let lo = self.data[index_block] >> index_offset;
            let mask = (1 << remaining_bits) - 1;
            let hi = self.data[index_block + 1] & mask;
            return (hi << fitting_bits) | lo;
        }

        let mask = (1 << width) - 1;
        (self.data[index_block] >> index_offset) & mask
    }

    /// Sets an integer of the given bit width at an index.
    ///
    /// This replaces the `width` bits at bit index `index width`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to write to.
    /// * `width` - The bit width of integers.
    ///
    /// # Safety
    ///
    /// If `n` is the amount of bits stored in this vector, then for `index` and `width`
    /// `(index + 1) * width < n` must hold.
    /// In addition, `value` must fit into `width` bits.
    ///
    unsafe fn set_unchecked_with_width(&mut self, index: usize, value: usize, width: usize) {
        let mask = (1 << width) - 1;
        let value = value & mask;
        let index_block = (index * width) / Self::block_width();
        let index_offset = (index * width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + width >= Self::block_width() {
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

    pub fn shrink_to_fit(&mut self) {
        let required_blocks = Self::num_required_blocks::<usize>(self.size, self.width);
        self.data.truncate(required_blocks);
        self.data.shrink_to_fit();
        self.recalculate_capacity();
    }
}

impl IntAccess for IntVec {
    fn get(&self, index: usize) -> usize {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }

        unsafe { self.get_unchecked(index) }
    }

    unsafe fn get_unchecked(&self, index: usize) -> usize {
        self.get_unchecked_with_width(index, self.width)
    }

    fn set(&mut self, index: usize, value: usize) {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }
        if value >= (1 << self.width) {
            panic!("value {value} too large for {}-bit integer", self.width)
        }
        unsafe { self.set_unchecked(index, value) }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: usize) {
        self.set_unchecked_with_width(index, value, self.width)
    }
}

impl IntoIterator for IntVec {
    type Item = usize;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

impl<'a> IntoIterator for &'a IntVec {
    type Item = usize;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { i: 0, v: self }
    }
}

pub struct IntoIter {
    i: usize,
    v: IntVec,
}

impl Iterator for IntoIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}

pub struct Iter<'a> {
    i: usize,
    v: &'a IntVec,
}

impl Iterator for Iter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}

#[cfg(test)]
mod test {
    use super::{traits::IntAccess, IntVec};

    #[test]
    fn basics_test() {
        let mut v = IntVec::new(4);
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
        let mut v = IntVec::new(23);
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
        let mut v = IntVec::new(7);
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
        let mut v = IntVec::new(7);
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
        let mut v = IntVec::new(8);

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
        let mut v = IntVec::new(12);
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
        let v = IntVec::new(7);
        v.get(10);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut v = IntVec::new(7);
        v.set(10, 10);
    }

    #[test]
    #[should_panic]
    fn set_too_large_number_test() {
        let mut v = IntVec::new(7);
        v.push(0);
        v.set(0, 100000000);
    }

    #[test]
    #[should_panic]
    fn push_too_large_number_test() {
        let mut v = IntVec::new(7);
        v.push(100000000);
    }

    #[test]
    fn bit_compress_test() {
        let mut v = IntVec::with_capacity(9, 25);

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
        let mut v = IntVec::with_capacity(9, 200);

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
