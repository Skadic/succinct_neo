use std::ops::{Bound, RangeBounds};

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

impl<B1: BitGet, B2: BitGet> PartialEq<BitSliceMut<'_, B2>> for BitSliceMut<'_, B1> {
    fn eq(&self, other: &BitSliceMut<'_, B2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        Iterator::eq(self.into_iter(), other.into_iter())
    }
}

impl<B1: BitGet, B2: BitGet> PartialEq<BitSliceMut<'_, B2>> for BitSlice<'_, B1> {
    fn eq(&self, other: &BitSliceMut<'_, B2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        Iterator::eq(self.into_iter(), other.into_iter())
    }
}

impl<B1: BitGet, B2: BitGet> PartialEq<BitSlice<'_, B2>> for BitSliceMut<'_, B1> {
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
        unsafe { self.get_unchecked(index) }
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
        unsafe { self.get_unchecked(index) }
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
        unsafe { self.set_unchecked(index, value) }
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
        unsafe { self.flip_unchecked(index) }
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

impl<T, R> SliceBit<R> for T
where
    T: BitGet,
    R: RangeBounds<usize>,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice(&self, r: R) -> BitSlice<Self> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.into_iter().len(),
        };

        BitSlice::new(self, start, end)
    }
}

impl<T, R> SliceBitMut<R> for T
where
    T: BitModify,
    R: RangeBounds<usize>,
    for<'a> &'a T: IntoIterator,
    for<'a> <&'a T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    fn slice_mut(&mut self, r: R) -> BitSliceMut<Self> {
        let start = match r.start_bound() {
            Bound::Excluded(&s) => s + 1,
            Bound::Included(&s) => s,
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Excluded(&e) => e,
            Bound::Included(&e) => e + 1,
            Bound::Unbounded => self.into_iter().len(),
        };

        BitSliceMut::new(self, start, end)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        bit_vec::BitVec,
        traits::{BitGet, BitModify, SliceBit, SliceBitMut},
    };

    #[test]
    fn full_range_test() {
        let mut bv = BitVec::new(80);
        let n = bv.len();
        let mut slice = bv.slice_mut(..);

        for i in 0..n {
            slice.set(i, i % 5 == 0);
        }

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
        let mut slice = bv.slice_mut(20..40);
        assert_eq!(20, slice.start, "incorrect mutable slice start");
        assert_eq!(40, slice.end, "incorrect mutable slice end");

        for i in 0..slice.len() {
            slice.set(i, i % 2 == 0);
        }

        let slice = bv.slice(20..40);
        assert_eq!(20, slice.start, "incorrect immutable slice start");
        assert_eq!(40, slice.end, "incorrect immutable slice end");
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
                (20..40).contains(&i) && i % 2 == 0,
                v,
                "incorrect value at index {i}"
            )
        }
    }

    #[test]
    fn range_inclusive_test() {
        let mut bv = BitVec::new(80);

        for i in 20..40 {
            bv.set(i, i % 2 == 0);
        }

        for (i, (expect, actual)) in bv
            .slice(20..40)
            .into_iter()
            .zip(bv.slice(20..=39))
            .enumerate()
        {
            assert_eq!(expect, actual, "incorrect value at index {} (immut)", i + 20)
        }

        let bv2 = bv.clone();
        for (i, (expect, actual)) in bv2
            .slice(20..40)
            .into_iter()
            .zip(bv.slice_mut(20..=39))
            .enumerate()
        {
            assert_eq!(expect, actual, "incorrect value at index {} (mut)", i + 20)
        }
    }

    #[test]
    fn range_to_test() {
        let mut bv = BitVec::new(80);

        for i in 20..40 {
            bv.set(i, i % 2 == 0);
        }

        for (i, (expect, actual)) in bv.slice(0..40).into_iter().zip(bv.slice(..40)).enumerate() {
            assert_eq!(expect, actual, "incorrect value at index {i}")
        }
    }

    #[test]
    fn range_to_inclusive_test() {
        let mut bv = BitVec::new(80);

        for i in 20..40 {
            bv.set(i, i % 2 == 0);
        }

        for (i, (expect, actual)) in bv.slice(0..40).into_iter().zip(bv.slice(..=39)).enumerate() {
            assert_eq!(expect, actual, "incorrect value at index {i}")
        }
    }

    #[test]
    fn range_from_test() {
        let mut bv = BitVec::new(80);

        for i in 20..40 {
            bv.set(i, i % 2 == 0);
        }

        for (i, (expect, actual)) in bv.slice(20..80).into_iter().zip(bv.slice(20..)).enumerate() {
            assert_eq!(expect, actual, "incorrect value at index {}", i + 20)
        }
    }

    #[test]
    fn get_test() {
        let mut bv = BitVec::new(80);
        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }
        let slice = bv.slice(10..70);
        for i in 0..slice.len() {
            assert_eq!(
                bv.get(i + 10),
                slice.get(i),
                "incorrect value at index {i} in immutable slice"
            )
        }

        let bv2 = bv.clone();
        let slice = bv.slice_mut(10..70);
        for i in 0..slice.len() {
            assert_eq!(
                bv2.get(i + 10),
                slice.get(i),
                "incorrect value at index {i} in immutable slice"
            )
        }
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_test() {
        let bv = BitVec::new(80);
        let slice = bv.slice(20..40);
        slice.get(20);
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_mut_test() {
        let mut bv = BitVec::new(80);
        let slice = bv.slice_mut(20..40);
        slice.get(20);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut bv = BitVec::new(80);
        let mut slice = bv.slice_mut(20..40);
        slice.set(20, true);
    }

    #[test]
    #[should_panic]
    fn flip_out_of_bounds_test() {
        let mut bv = BitVec::new(80);
        let mut slice = bv.slice_mut(20..40);
        slice.flip(20);
    }

    #[test]
    fn set_test() {
        let mut bv = BitVec::new(80);
        let mut slice = bv.slice_mut(..);
        for i in 0..slice.len() {
            slice.set(i, i % 2 == 0)
        }

        let slice = bv.slice(10..70);
        for i in 0..slice.len() {
            assert_eq!(
                bv.get(i + 10),
                slice.get(i),
                "incorrect value at index {i} in immutable slice"
            )
        }
    }

    #[test]
    fn flip_test() {
        let mut bv = BitVec::new(80);
        let mut slice = bv.slice_mut(..);
        for i in 0..slice.len() {
            slice.set(i, i % 2 == 0)
        }
        for i in 0..slice.len() {
            slice.flip(i)
        }

        for i in 0..bv.len() {
            assert_eq!(i % 2 != 0, bv.get(i), "incorrect value at index {i}")
        }
    }

    #[test]
    fn iter_test() {
        let mut bv = BitVec::new(80);
        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }

        let slice = bv.slice(20..80);
        for (i, v) in (&slice).into_iter().enumerate() {
            assert_eq!(
                i % 2 == 0,
                v,
                "incorrect value at index {} (immut ref)",
                i + 20
            )
        }
        for (i, v) in slice.into_iter().enumerate() {
            assert_eq!(i % 2 == 0, v, "incorrect value at index {} (immut)", i + 20)
        }

        let slice = bv.slice_mut(20..80);
        for (i, v) in (&slice).into_iter().enumerate() {
            assert_eq!(
                i % 2 == 0,
                v,
                "incorrect value at index {} (mut ref)",
                i + 20
            )
        }
        for (i, v) in slice.into_iter().enumerate() {
            assert_eq!(i % 2 == 0, v, "incorrect value at index {} (mut)", i + 20)
        }
    }

    #[test]
    fn equality_test() {
        let mut bv = BitVec::new(80);
        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }

        let mut bv2 = bv.clone();

        let s1 = bv.slice(10..50);
        let s2 = bv2.slice(20..60);
        assert_eq!(s1, s2, "immutable-immutable slices not equal");
        let s2 = bv2.slice(20..70);
        assert_ne!(s1, s2, "immutable-immutable slices are equal");

        let s1 = bv.slice(30..50);
        let s2 = bv2.slice_mut(60..80);
        assert_eq!(s1, s2, "immutable-mutable slices not equal");
        assert_eq!(s2, s1, "mutable-immutable slices not equal");
        let s2 = bv2.slice_mut(60..70);
        assert_ne!(s1, s2, "immutable-mutable slices are equal");
        assert_ne!(s2, s1, "mutable-immutable slices are equal");

        let s1 = bv.slice_mut(30..50);
        let s2 = bv2.slice_mut(60..80);
        assert_eq!(s1, s2, "mutable-mutable slices not equal");
        let s2 = bv2.slice_mut(60..70);
        assert_ne!(s1, s2, "mutable-mutable slices are equal");
    }
}
