use crate::traits::BitAccess;

/// The word size on this machine in bits
const WORD_SIZE: usize = 64;

/// The logarithm of the word size for dividing by the word size quickly
const WORD_EXP: usize = 6;

/// A mask for quickly calculating the modulus
const WORD_MASK: usize = (1 << WORD_EXP) - 1;

pub struct BitVec {
    data: Vec<usize>,
    size: usize,
}

impl BitVec {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0;(size as f64 / WORD_SIZE as f64).ceil() as usize],
            size
        }
    }
}

impl BitAccess for BitVec {
    fn len(&self) -> usize {
        self.size
    }

    unsafe fn get_unchecked(&self, index: usize) -> bool {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;
        unsafe { self.data.get_unchecked(block_index) & (1 << internal_index) > 0 }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: bool) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        let block = unsafe { self.data.get_unchecked_mut(block_index) };

        if value {
            *block |= 1 << internal_index;
        } else {
            *block &= 0 << internal_index;
        }
    }

    unsafe fn flip_unchecked(&mut self, index: usize) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        unsafe { *self.data.get_unchecked_mut(block_index) ^= 1 << internal_index }
    }
}

mod test {
    use crate::traits::BitAccess;
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
}