use crate::traits::BitGet;

mod trait_impls;

/// A view into a segment of a type which supports `BitGet` and `BitModify` if the backing type supports it respectively.
/// 
/// Properties:
/// 
/// * `backing`: The backing store for the bits.
/// * `start`: The index of the first bit in the slice.
/// * `end`: The index of the first bit that is not part of the slice.
/// 
/// # Examples
/// 
/// ```
/// use succinct_neo::bit_vec::{BitVec, slice::BitSliceMut};
/// use succinct_neo::traits::{BitGet, BitModify, SliceBitMut};
/// 
/// let mut bv = BitVec::new(16);
/// let mut slice = bv.slice_mut(8..10);
/// assert_eq!(2, slice.len());
/// 
/// slice.set(0, true);
/// // We can't access the original bitvector if the (mutably borrowing) slice is still around.
/// drop(slice);
/// 
/// assert_eq!(true, bv.get(8));
/// ```
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

    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Backing: BitGet> BitSlice<'a, Backing> {
    pub fn iter(&self) -> Iter<&'a Backing> {
        Iter {
            backing: self.backing,
            current: 0,
            end: self.len(),
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

    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct Iter<Backing> {
    backing: Backing,
    current: usize,
    end: usize,
}

impl<Backing> Iter<Backing> {
    pub fn new(backing: Backing, start: usize, end: usize) -> Self {
        Self {
            backing,
            current: start,
            end,
        }
    }
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
                (20..40).contains(&i) && i % 2 == 0,
                v,
                "incorrect value at index {i}"
            )
        }
    }
}
