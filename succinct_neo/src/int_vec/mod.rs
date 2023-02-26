use std::marker::PhantomData;
pub use self::traits::IntAccess;

mod traits;
mod dynamic;

#[derive(Debug)]
pub struct Dynamic;
#[derive(Debug)]
pub struct Fixed<const WIDTH: usize>;


#[derive(Debug)]
pub struct IntVec<IntWidth> {
    data: Vec<usize>,
    width: usize,
    capacity: usize,
    size: usize,
    _marker: PhantomData<IntWidth>
}

impl<T> IntVec<T> {

    #[inline]
    const fn block_width() -> usize {
        std::mem::size_of::<usize>() * 8
    }

    /// Returns the amount of integers would fit into the currently allocated memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let v = IntVec::with_capacity(5, 50);
    ///
    /// // 50 integers of 5 bit each, would fit into 250 bits in total which would make 4 * 64 bit
    /// // blocks, making 256 bits in total. However, 256 bits fit 51 integers of size 5.
    /// assert_eq!(51, v.capacity());
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

}

#[cfg(test)]
mod test {
    use crate::int_vec::Dynamic;
    use super::{traits::IntAccess, IntVec};

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
