use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::traits::{BitGet, BitModify, SliceBit, SliceBitMut};

use super::{BitSlice, BitSliceMut, Iter};

impl<B1: BitGet, B2: BitGet> PartialEq<BitSlice<'_, B2>> for BitSlice<'_, B1> {
    fn eq(&self, other: &BitSlice<'_, B2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        Iterator::eq(self.into_iter(), other.into_iter())
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

impl<Backing: BitGet> Iterator for Iter<Backing> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let v = unsafe { self.backing.get_unchecked(self.current) };
        self.current += 1;
        Some(v)
    }
}

impl<Backing: BitGet> ExactSizeIterator for Iter<Backing> {
    fn len(&self) -> usize {
        self.end - self.current
    }
}

impl<'a, Backing: BitGet> IntoIterator for BitSlice<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<&'a Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.backing, self.start, self.end)
    }
}

impl<'a, Backing: BitGet> IntoIterator for &'_ BitSlice<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<&'a Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.backing, self.start, self.end)
    }
}

impl<'a, Backing: BitGet> IntoIterator for BitSliceMut<'a, Backing> {
    type Item = bool;

    type IntoIter = Iter<&'a mut Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.backing, self.start, self.end)
    }
}

impl<'a, 'b, Backing: BitGet> IntoIterator for &'a BitSliceMut<'b, Backing> {
    type Item = bool;

    type IntoIter = Iter<&'a &'b mut Backing>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(&self.backing, self.start, self.end)
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

impl<T> SliceBit<RangeToInclusive<usize>> for T
where
    T: BitGet,
{
    fn slice(&self, range: RangeToInclusive<usize>) -> BitSlice<Self> {
        BitSlice::new(self, 0, range.end + 1)
    }
}

impl<T> SliceBit<RangeInclusive<usize>> for T
where
    T: BitGet,
{
    fn slice(&self, range: RangeInclusive<usize>) -> BitSlice<Self> {
        BitSlice::new(self, *range.start(), *range.end() + 1)
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
        BitSliceMut::new(self, 0, range.end + 1)
    }
}

impl<T> SliceBitMut<RangeToInclusive<usize>> for T
where
    T: BitModify,
{
    fn slice_mut(&mut self, range: RangeToInclusive<usize>) -> BitSliceMut<Self> {
        BitSliceMut::new(self, 0, range.end + 1)
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

