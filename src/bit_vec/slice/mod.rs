use core::panic;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo};

use crate::traits::{BitGet, BitModify, SliceBit, SliceBitMut};

mod trait_impls;

pub struct BitSlice<'a, Backing> {
    backing: &'a Backing,
    start: usize,
    end: usize,
}

impl<'a, Backing> BitSlice<'a, Backing> {
    pub fn new(backing: &'a Backing, start: usize, end: usize) -> Self {
        debug_assert!(
            start <= end,
            "end index must be greater or equal to the start index"
        );
        Self {
            backing,
            start,
            end,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<'a, Backing: BitGet> BitSlice<'a, Backing> {
    pub fn iter(&self) -> Iter<'a, Backing> {
        Iter {
            backing: self.backing,
            len: self.len(),
            current: 0,
        }
    }
}

pub struct BitSliceMut<'a, Backing> {
    backing: &'a mut Backing,
    start: usize,
    end: usize,
}

impl<'a, Backing> BitSliceMut<'a, Backing> {
    pub fn new(backing: &'a mut Backing, start: usize, end: usize) -> Self {
        debug_assert!(
            start <= end,
            "end index must be greater or equal to the start index"
        );
        Self {
            backing,
            start,
            end,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

pub struct Iter<'a, Backing> {
    backing: &'a Backing,
    len: usize,
    current: usize,
}

#[cfg(test)]
mod test {
    use crate::{
        bit_vec::BitVec,
        traits::{BitModify, SliceBit, SliceBitMut},
    };

    #[test]
    fn full_range_test() {
        let mut bv = BitVec::new(80);
        let n = bv.len();
        let mut slice = bv.slice_mut(..);

        for i in 0..n {
            slice.set(i, i % 5 == 0);
        }
        drop(slice);

        let slice = bv.slice(..);
        for (i, (expect, actual)) in bv.iter().zip(slice).enumerate() {
            assert_eq!(
                expect, actual,
                "incorrect value at immutable slice index {i}"
            )
        }

        for (i, v) in bv.into_iter().enumerate() {
            assert_eq!(i % 5 == 0, v, "incorrect value at index {i}")
        }
    }

    #[test]
    fn range_test() {
        let mut bv = BitVec::new(80);
        let n = bv.len();
        let mut slice = bv.slice_mut(20..40);

        for i in 0..slice.len() {
            slice.set(i, i % 2 == 0);
        }
        drop(slice);

        let slice = bv.slice(20..40);
        for (i, (expect, actual)) in bv.iter().skip(20).zip(slice).enumerate() {
            assert_eq!(
                expect,
                actual,
                "incorrect value at immutable slice index {}",
                i + 20
            )
        }

        for (i, v) in bv.into_iter().enumerate() {
            assert_eq!(
                if i >= 20 && i < 40 { i % 5 == 0 } else { false },
                v,
                "incorrect value at index {i}"
            )
        }
    }
}
