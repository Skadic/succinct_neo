use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use itertools::Itertools;

pub use traits::*;

use crate::bit_vec::slice::BitSlice;

use self::slice::Iter;

/// Trait implementations for the backing type of BitVec
mod backing;
/// Bit slices offering views into types that offer bit access
pub mod slice;
pub mod traits;

/// The word size on this machine in bits
const WORD_SIZE: usize = 64;

/// The logarithm of the word size for multiplying/dividing by the word size quickly
const WORD_EXP: usize = 6;

/// A mask for quickly calculating the modulus
const WORD_MASK: usize = (1 << WORD_EXP) - 1;

///
/// A fixed-size bit vector allocated on the heap.
///
/// # Examples
///
/// ```
/// use succinct_neo::bit_vec::{BitVec, BitModify, BitGet};
///
/// // A bit vector with space for 16 bits
/// let mut bv = BitVec::new(16);
///
/// // Views into the bit vector can be retrieved through slices
/// let mut slice = bv.slice_mut(4..8);
///
/// for i in 0..slice.len() {
///     slice.set_bit(i, true);
/// }
///
/// for i in 0..bv.len() {
///     assert_eq!(4 <= i && i < 8, bv.get_bit(i))
/// }
/// ```
///
#[derive(Clone)]
pub struct BitVec {
    data: BitSlice<Box<[usize]>>,
    size: usize,
}

impl BitVec {
    /// Creates a new [`BitVec`].
    ///
    /// # Arguments
    ///
    /// * `size`: The size of this bitvector.
    ///
    /// returns: A new bit vector with all indices set to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::bit_vec::BitVec;
    ///
    /// // A bit vector with space for 16 bits
    /// let bv = BitVec::new(16);
    /// ```
    pub fn new(size: usize) -> Self {
        let v = vec![0usize; (size as f64 / WORD_SIZE as f64).ceil() as usize];
        let b = v.into_boxed_slice();
        Self {
            data: BitSlice::new(b, 0, size),
            size,
        }
    }

    pub fn raw(&self) -> &[usize] {
        self.data.backing()
    }
}

impl BitModify for BitVec {
    #[inline]
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        self.data.set_bit_unchecked(index, value)
    }

    #[inline]
    fn set_bit(&mut self, index: usize, value: bool) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.size)
        }
        unsafe { self.set_bit_unchecked(index, value) }
    }

    #[inline]
    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        self.data.flip_bit_unchecked(index)
    }

    #[inline]
    fn flip_bit(&mut self, index: usize) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.size)
        }
        unsafe { self.flip_bit_unchecked(index) }
    }
}

impl<'a> IntoIterator for &'a BitVec {
    type Item = bool;

    type IntoIter = Iter<&'a Box<[usize]>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl Debug for BitVec {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")
            .and_then(|_| {
                write!(
                    f,
                    "{}",
                    self.iter()
                        .map(|v| if v { "1" } else { "0" })
                        .intersperse(", ")
                        .collect::<String>()
                )
            })
            .and_then(|_| write!(f, "}}"))
    }
}

impl Deref for BitVec {
    type Target = BitSlice<Box<[usize]>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for BitVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl AsRef<BitSlice<Box<[usize]>>> for BitVec {
    fn as_ref(&self) -> &BitSlice<Box<[usize]>> {
        &self.data
    }
}

impl AsRef<[usize]> for BitVec {
    fn as_ref(&self) -> &[usize] {
        self.data.backing()
    }
}

impl AsMut<BitSlice<Box<[usize]>>> for BitVec {
    fn as_mut(&mut self) -> &mut BitSlice<Box<[usize]>> {
        &mut self.data
    }
}

#[cfg(test)]
mod test {
    use crate::bit_vec::BitGet;
    use crate::bit_vec::slice::BitSlice;

    use super::BitVec;
    use super::traits::BitModify;

    #[test]
    fn basics_test() {
        let bv = BitVec::new(80);
        assert_eq!(80, bv.len(), "length incorrect");
        assert!(!bv.is_empty(), "bv empty despite length being 80");
        let bv = BitVec::new(0);
        assert_eq!(0, bv.len(), "length incorrect");
        assert!(bv.is_empty(), "bv not empty despite length being 0");

        let mut bv = BitVec::new(80);
        bv.set(10, true);

        assert_eq!(bv.backing(), AsRef::<BitSlice<_>>::as_ref(&bv).backing());
        assert_eq!(bv.raw(), AsRef::<[usize]>::as_ref(&bv));
        assert_eq!(bv.backing(), bv.clone().as_mut().backing());

        println!("{bv:?}")
    }

    #[test]
    fn set_get_test() {
        let mut bv = BitVec::new(160);
        for i in (0..bv.len()).step_by(3) {
            bv.set(i, true);
        }

        for i in 0..bv.len() {
            assert_eq!(i % 3 == 0, bv.get(i));
        }
    }

    #[test]
    fn set_get_bit_test() {
        let mut bv = BitVec::new(160);
        for i in (0..bv.len()).step_by(3) {
            bv.set_bit(i, true);
        }

        for i in 0..bv.len() {
            assert_eq!(i % 3 == 0, bv.get_bit(i));
        }
    }

    #[test]
    fn flip_test() {
        let mut bv = BitVec::new(160);
        for i in (0..bv.len()).step_by(3) {
            bv.set(i, true);
        }

        for i in 0..bv.len() {
            bv.flip(i);
        }

        for i in 0..bv.len() {
            assert_eq!(i % 3 != 0, bv.get_bit(i));
        }
    }

    #[test]
    fn flip_bit_test() {
        let mut bv = BitVec::new(160);
        for i in (0..bv.len()).step_by(3) {
            bv.set_bit(i, true);
        }

        for i in 0..bv.len() {
            bv.flip_bit(i);
        }

        for i in 0..bv.len() {
            assert_eq!(i % 3 != 0, bv.get_bit(i));
        }
    }

    #[test]
    fn into_iter_test() {
        let mut bv = BitVec::new(160);
        let n = bv.size;
        for i in (0..bv.len()).step_by(3) {
            bv.set(i, true);
        }

        for i in 0..bv.len() {
            bv.flip(i);
        }

        let iter = bv.into_iter();
        assert_eq!(n, iter.len(), "incorrect len stored in iter");

        for (i, v) in iter.enumerate() {
            assert_eq!(i % 3 != 0, v);
        }
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_mut_test() {
        let bv = BitVec::new(20);
        bv.get(20);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut bv = BitVec::new(20);
        bv.set(20, true);
    }

    #[test]
    #[should_panic]
    fn flip_out_of_bounds_test() {
        let mut bv = BitVec::new(20);
        bv.flip(20);
    }

    #[test]
    #[should_panic]
    fn get_bit_out_of_bounds_mut_test() {
        let bv = BitVec::new(20);
        bv.get_bit(20);
    }

    #[test]
    #[should_panic]
    fn set_bit_out_of_bounds_test() {
        let mut bv = BitVec::new(20);
        bv.set_bit(20, true);
    }

    #[test]
    #[should_panic]
    fn flip_bit_out_of_bounds_test() {
        let mut bv = BitVec::new(20);
        bv.flip_bit(20);
    }
}
