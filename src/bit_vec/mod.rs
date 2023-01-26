use crate::traits::{BitGet, BitModify};
use std::fmt::{Debug, Formatter};

use self::slice::Iter;
use itertools::Itertools;

pub mod slice;

/// The word size on this machine in bits
const WORD_SIZE: usize = 64;

/// The logarithm of the word size for dividing by the word size quickly
const WORD_EXP: usize = 6;

/// A mask for quickly calculating the modulus
const WORD_MASK: usize = (1 << WORD_EXP) - 1;

#[derive(Clone, PartialEq, Eq)]
pub struct BitVec {
    data: Vec<usize>,
    size: usize,
}

impl BitVec {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; (size as f64 / WORD_SIZE as f64).ceil() as usize],
            size,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn iter(&self) -> Iter<&Self> {
        self.into_iter()
    }
}

impl BitGet for BitVec {
    unsafe fn get_unchecked(&self, index: usize) -> bool {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;
        unsafe { self.data.get_unchecked(block_index) & (1 << internal_index) > 0 }
    }

    fn get(&self, index: usize) -> bool {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.size)
        }
        unsafe { self.get_unchecked(index) }
    }
}

impl BitModify for BitVec {
    unsafe fn set_unchecked(&mut self, index: usize, value: bool) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        let block = unsafe { self.data.get_unchecked_mut(block_index) };

        if value {
            *block |= 1 << internal_index;
        } else {
            *block &= !(1 << internal_index);
        }
    }

    fn set(&mut self, index: usize, value: bool) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.size)
        }
        unsafe { self.set_unchecked(index, value) }
    }

    unsafe fn flip_unchecked(&mut self, index: usize) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        unsafe { *self.data.get_unchecked_mut(block_index) ^= 1 << internal_index }
    }

    fn flip(&mut self, index: usize) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.size)
        }
        unsafe { self.flip_unchecked(index) }
    }
}

pub struct IntoIter {
    bit_vec: BitVec,
    current: usize,
}

impl Iterator for IntoIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.bit_vec.size {
            return None;
        }
        let v = self.bit_vec.get(self.current);
        self.current += 1;
        Some(v)
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.bit_vec.size - self.current
    }
}

impl IntoIterator for BitVec {
    type Item = bool;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            bit_vec: self,
            current: 0,
        }
    }
}

impl<'a> IntoIterator for &'a BitVec {
    type Item = bool;

    type IntoIter = Iter<&'a BitVec>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self, 0, self.size)
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

#[cfg(test)]
mod test {
    use crate::traits::{BitGet, BitModify};

    use super::BitVec;

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
    fn flip_test() {
        let mut bv = BitVec::new(160);
        for i in (0..bv.len()).step_by(3) {
            bv.set(i, true);
        }

        for i in 0..bv.len() {
            bv.flip(i);
        }

        for i in 0..bv.len() {
            assert_eq!(i % 3 != 0, bv.get(i));
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
}
