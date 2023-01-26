use core::panic;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo};

use crate::traits::{BitGet, BitModify, SliceBit, SliceBitMut};

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

impl<Backing: BitGet> BitGet for BitSlice<'_, Backing> {
    unsafe fn get_unchecked(&self, index: usize) -> bool {
        self.backing.get_unchecked(self.start + index)
    }

    fn get(&self, index: usize) -> bool {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }
        unsafe { self.backing.get_unchecked(self.start + index) }
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

impl<Backing: BitGet> BitGet for BitSliceMut<'_, Backing> {
    #[inline]
    unsafe fn get_unchecked(&self, index: usize) -> bool {
        self.backing.get_unchecked(self.start + index)
    }

    #[inline]
    fn get(&self, index: usize) -> bool {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }
        unsafe { self.backing.get_unchecked(self.start + index) }
    }
}

impl<Backing: BitModify> BitModify for BitSliceMut<'_, Backing> {
    #[inline]
    unsafe fn set_unchecked(&mut self, index: usize, value: bool) {
        self.backing.set_unchecked(self.start + index, value)
    }

    #[inline]
    fn set(&mut self, index: usize, value: bool) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }
        unsafe { self.backing.set_unchecked(self.start + index, value) }
    }

    #[inline]
    unsafe fn flip_unchecked(&mut self, index: usize) {
        self.backing.flip_unchecked(self.start + index)
    }

    #[inline]
    fn flip(&mut self, index: usize) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }
        unsafe { self.backing.flip_unchecked(self.start + index) }
    }
}

pub struct Iter<'a, Backing> {
    backing: &'a Backing,
    len: usize,
    current: usize,
}

impl<'a, Backing: BitGet> Iterator for Iter<'a, Backing> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.len {
            return None;
        }
        let v = unsafe { self.backing.get_unchecked(self.current) };
        self.current += 1;
        Some(v)
    }
}

impl<'a, Backing: BitGet> ExactSizeIterator for Iter<'a, Backing> {
    fn len(&self) -> usize {
        self.len - self.current
    }
}

impl<'a, Backing: BitGet> IntoIterator for BitSlice<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<'a, Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            backing: self.backing,
            len: self.len(),
            current: 0,
        }
    }
}

impl<'a, Backing: BitGet> IntoIterator for &BitSlice<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<'a, Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            backing: self.backing,
            len: self.len(),
            current: 0,
        }
    }
}

impl<'a, Backing: BitGet> IntoIterator for BitSliceMut<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<'a, Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            backing: self.backing,
            len: self.len(),
            current: 0,
        }
    }
}

impl<'a, Backing: BitGet> IntoIterator for &'a BitSliceMut<'_, Backing> {
    type Item = bool;

    type IntoIter = Iter<'a, Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            backing: self.backing,
            len: self.len(),
            current: 0,
        }
    }
}

// ------------------ SLICE BITS ------------------

impl<T> SliceBit<RangeFull> for T
where
    T: BitGet,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice(&self, _: RangeFull) -> BitSlice<Self> {
        BitSlice::new(self, 0, self.into_iter().len())
    }
}

impl<T> SliceBit<RangeFrom<usize>> for T
where
    T: BitGet,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice(&self, range: RangeFrom<usize>) -> BitSlice<Self> {
        BitSlice::new(self, range.start, self.into_iter().len())
    }
}

impl<T> SliceBit<Range<usize>> for T
where
    T: BitGet,
{
    fn slice(&self, range: Range<usize>) -> BitSlice<Self> {
        BitSlice::new(self, range.start, range.end)
    }
}

impl<T> SliceBit<RangeTo<usize>> for T
where
    T: BitGet,
{
    fn slice(&self, range: RangeTo<usize>) -> BitSlice<Self> {
        BitSlice::new(self, 0, range.end)
    }
}

impl<T> SliceBit<RangeInclusive<usize>> for T
where
    T: BitGet,
{
    fn slice(&self, range: RangeInclusive<usize>) -> BitSlice<Self> {
        BitSlice::new(self, *range.start(), *range.end())
    }
}

impl<T> SliceBitMut<RangeFull> for T
where
    T: BitModify,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice_mut(&mut self, _: RangeFull) -> BitSliceMut<Self> {
        let len = self.into_iter().len();
        BitSliceMut::new(self, 0, len)
    }
}

impl<T> SliceBitMut<RangeFrom<usize>> for T
where
    T: BitModify,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice_mut(&mut self, range: RangeFrom<usize>) -> BitSliceMut<Self> {
        let len = self.into_iter().len();
        BitSliceMut::new(self, range.start, len)
    }
}

impl<T> SliceBitMut<Range<usize>> for T
where
    T: BitModify,
{
    fn slice_mut(&mut self, range: Range<usize>) -> BitSliceMut<Self> {
        BitSliceMut::new(self, range.start, range.end)
    }
}

impl<T> SliceBitMut<RangeTo<usize>> for T
where
    T: BitModify,
{
    fn slice_mut(&mut self, range: RangeTo<usize>) -> BitSliceMut<Self> {
        BitSliceMut::new(self, 0, range.end)
    }
}

impl<T> SliceBitMut<RangeInclusive<usize>> for T
where
    T: BitModify,
{
    fn slice_mut(&mut self, range: RangeInclusive<usize>) -> BitSliceMut<Self> {
        BitSliceMut::new(self, *range.start(), *range.end())
    }
}

#[cfg(test)]
mod test {
    use crate::{bit_vec::BitVec, traits::{SliceBit, SliceBitMut, BitModify}};

    #[test]
    fn full_range_test() {
        let mut bv = BitVec::new(80);
        let n = bv.len();
        let mut slice = bv.slice_mut(..);

        for i in 0..n {
            slice.set(i, i % 5 == 0);
        }
        drop(slice);

        for (i, v) in bv.into_iter().enumerate() {
            assert_eq!(i % 5 == 0, v, "incorrect value at index {i}")
        }
    }
}
