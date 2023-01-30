use crate::traits::{BitGet, BitModify};

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
/// use succinct_neo::bit_vec::{BitVec, slice::BitSlice};
/// use succinct_neo::traits::{BitGet, BitModify, SliceBit};
///
/// let mut bv = BitVec::new(16);
/// let mut slice = bv.slice_bits_mut(8..10);
/// assert_eq!(2, slice.len());
///
/// slice.set_bit(0, true);
/// // We can't access the original bitvector if the (mutably borrowing) slice is still around.
/// drop(slice);
///
/// assert_eq!(true, bv.get_bit(8));
/// ```
#[derive(Debug, Clone)]
pub struct BitSlice<Backing> {
    backing: Backing,
    start: usize,
    end: usize,
}

impl<Backing> BitSlice<Backing> {
    pub fn new(backing: Backing, start: usize, end: usize) -> Self {
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

    #[inline]
    pub fn backing(&self) -> &Backing {
        &self.backing
    }
}

impl<Backing: BitGet> BitSlice<Backing> {
    pub fn iter(&self) -> Iter<&Backing> {
        Iter {
            backing: &self.backing,
            current: self.start,
            end: self.end,
        }
    }

    pub fn split_at(&self, index: usize) -> (BitSlice<&Backing>, BitSlice<&Backing>) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }

        (
            BitSlice::new(&self.backing, self.start, self.start + index),
            BitSlice::new(&self.backing, self.start + index, self.end),
        )
    }
}

impl<Backing: BitModify> BitSlice<Backing> {
    pub fn split_at_mut(
        &mut self,
        index: usize,
    ) -> (BitSlice<&mut Backing>, BitSlice<&mut Backing>) {
        if index >= self.len() {
            panic!("index is {index} but length is {}", self.len())
        }

        let ptr = &mut self.backing as *mut Backing;

        // SAFETY: We know this is safe since we just created the pointer so it definitely is not null
        // Also, since the slices we create do not overlap, it is no problem to have two mutable references to the backing datastructure
        unsafe {
            (
                BitSlice::new(ptr.as_mut().unwrap(), self.start, self.start + index),
                BitSlice::new(ptr.as_mut().unwrap(), self.start + index, self.end),
            )
        }
    }
}

#[derive(Debug)]
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
        traits::{BitModify, SliceBit},
    };

    use super::BitSlice;

    #[test]
    fn is_empty_test() {
        let mut bv = BitVec::new(80);

        let slice = bv.slice_bits(40..40);
        assert_eq!(0, slice.len(), "immutable slice not empty");
        assert!(slice.is_empty(), "immutable slice not empty");

        let slice = bv.slice_bits_mut(40..40);
        assert_eq!(0, slice.len(), "mutable slice not empty");
        assert!(slice.is_empty(), "mutable slice not empty")
    }

    #[test]
    fn iter_test() {
        let mut bv = BitVec::new(80);

        let mut slice = bv.slice_bits_mut(20..40);
        for i in 0..slice.len() {
            slice.set_bit(i, (i / 5) % 2 == 0)
        }

        for (i, actual) in slice.iter().enumerate() {
            assert_eq!(
                (i / 5) % 2 == 0,
                actual,
                "incorrect value in mutable slice at {}",
                i + 20
            )
        }

        let slice = bv.slice_bits(20..40);
        for (i, actual) in slice.iter().enumerate() {
            assert_eq!(
                (i / 5) % 2 == 0,
                actual,
                "incorrect value in immutable slice {}",
                i + 20
            )
        }
    }

    #[test]
    #[should_panic]
    fn slice_invalid_bound_test() {
        let bv = BitVec::new(80);
        BitSlice::new(&bv, 10, 9);
    }

    #[test]
    fn debug_test() {
        let mut bv = BitVec::new(80);
        let slice = bv.slice_bits_mut(20..40);

        println!("{slice:?}");
        let slice = bv.slice_bits(10..50);
        println!("{slice:?}");
        println!("{:?}", bv.iter());
    }

    #[test]
    fn split_test() {
        let mut bv = BitVec::new(80);
        for i in 0..bv.len() {
            bv.set_bit(i, i % 2 == 0)
        }
        let slice = bv.slice_bits(20..40);

        let mut bv2 = bv.clone();

        let (l, r) = slice.split_at(10);
        let slice_left = bv.slice_bits(20..30);
        let slice_right = bv.slice_bits(30..40);
        assert_eq!(
            slice_left, l,
            "left-split part of immutable slice not the same"
        );
        assert_eq!(
            slice_right, r,
            "right-split part of immutable slice not the same"
        );

        let mut slice = bv.slice_bits_mut(20..40);

        let (l, r) = slice.split_at(10);
        let slice_left = bv2.slice_bits(20..30);
        let slice_right = bv2.slice_bits(30..40);
        assert_eq!(
            slice_left, l,
            "left-split part of mutable slice not the same"
        );
        assert_eq!(
            slice_right, r,
            "right-split part of mutable slice not the same"
        );

        let (l, r) = slice.split_at_mut(10);
        let slice_left = bv2.slice_bits_mut(20..30);
        assert_eq!(
            slice_left, l,
            "mutable left-split part of mutable slice not the same"
        );
        let slice_right = bv2.slice_bits_mut(30..40);
        assert_eq!(
            slice_right, r,
            "mutable right-split part of mutable slice not the same"
        );
    }
}
