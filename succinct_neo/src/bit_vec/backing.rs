use crate::traits::{BitGet, BitModify};

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
