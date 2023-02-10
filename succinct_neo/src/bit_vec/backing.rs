use super::{BitGet, BitModify};
use super::{WORD_EXP, WORD_MASK};

impl BitGet for [usize] {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;
        unsafe { self.get_unchecked(block_index) & (1 << internal_index) > 0 }
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        if index >= self.len() << WORD_EXP {
            panic!("index is {index} but length is {}", self.len() << WORD_EXP)
        }
        unsafe { self.get_bit_unchecked(index) }
    }
}

impl BitModify for [usize] {
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        let block = unsafe { self.get_unchecked_mut(block_index) };

        if value {
            *block |= 1 << internal_index;
        } else {
            *block &= !(1 << internal_index);
        }
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        if index >= self.len() << WORD_EXP {
            panic!("index is {index} but length is {}", self.len() << WORD_EXP)
        }
        unsafe { self.set_bit_unchecked(index, value) }
    }

    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        let block_index = index >> WORD_EXP;
        let internal_index = index & WORD_MASK;

        unsafe { *self.get_unchecked_mut(block_index) ^= 1 << internal_index }
    }

    fn flip_bit(&mut self, index: usize) {
        if index >= self.len() << WORD_EXP {
            panic!("index is {index} but length is {}", self.len() << WORD_EXP)
        }
        unsafe { self.flip_bit_unchecked(index) }
    }
}

#[cfg(test)]
mod test {
    use crate::bit_vec::{BitGet, BitModify};

    #[test]
    fn op_test() {
        let mut slice = [0b1100_1100_1010_1010usize];

        for i in 0..slice.len() {
            slice.set_bit(i, if i < 8 { i % 2 == 1 } else { (i / 2) % 2 == 1 })
        }

        for i in 0..slice.len() {
            assert_eq!(if i < 8 { i % 2 == 1 } else { (i / 2) % 2 == 1 }, slice.get_bit(i))
        }

        for i in 0..slice.len() {
            slice.flip_bit(i)
        }

        for i in 0..slice.len() {
            assert_eq!(if i < 8 { i % 2 == 0 } else { (i / 2) % 2 == 0 }, slice.get_bit(i))
        }
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds_test() {
        let slice = [0b1100_1100_1010_1010usize];
        slice.get_bit(100);
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds_test() {
        let mut slice = [0b1100_1100_1010_1010usize];
        slice.set_bit(100, true);
    }

    #[test]
    #[should_panic]
    fn flip_out_of_bounds_test() {
        let mut slice = [0b1100_1100_1010_1010usize];
        slice.flip_bit(100);
    }
}