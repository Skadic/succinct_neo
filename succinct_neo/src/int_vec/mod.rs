use core::panic;

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
    #[inline]
    pub fn new(width: usize) -> Self {
        Self::with_capacity(width, 8)
    }

    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        let block_size = Self::block_width();
        let num_blocks = (capacity * width) / block_size;

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width,
            capacity: num_blocks * block_size / width,
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
            self.capacity = self.data.capacity() * Self::block_width() / self.width;
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
    fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter { i: 0, v: self }
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
        let index_block = (index * self.width) / Self::block_width();
        let index_offset = (index * self.width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + self.width >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            let remaining_bits = self.width - fitting_bits;
            let lo = self.data[index_block] >> index_offset;
            let mask = (1 << remaining_bits) - 1;
            let hi = self.data[index_block + 1] & mask;
            return (hi << fitting_bits) | lo;
        }

        let mask = (1 << self.width) - 1;
        (self.data[index_block] >> index_offset) & mask
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
        let mask = (1 << self.width) - 1;
        let value = value & mask;
        let index_block = (index * self.width) / Self::block_width();
        let index_offset = (index * self.width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + self.width >= Self::block_width() {
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
}
