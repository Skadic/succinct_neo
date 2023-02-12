use super::{BitGet, BitModify};

use super::{WORD_EXP, WORD_MASK};

impl BitGet for usize {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        self & (1 << index) > 0
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        if index >= std::mem::size_of::<Self>() * 8 {
            panic!("index is {index} but length is {}", std::mem::size_of::<Self>() * 8)
        }

        // SAFETY: We checked the index is in bounds
        unsafe { self.get_bit_unchecked(index) }
    }
}

impl BitModify for usize {
    #[inline]
    unsafe fn set_bit_unchecked(&mut self, index: usize, value: bool) {
        if value {
            *self |= 1 << index
        } else {
            *self &= !(1 << index)
        }
    }

    #[inline]
    fn set_bit(&mut self, index: usize, value: bool) {
        if index >= std::mem::size_of::<Self>() * 8 {
            panic!("index is {index} but length is {}", std::mem::size_of::<Self>() * 8)
        }
        // SAFETY: We checked the index is in bounds
        unsafe { self.set_bit_unchecked(index, value) }
    }

    #[inline]
    unsafe fn flip_bit_unchecked(&mut self, index: usize) {
        *self ^= 1 << index
    }

    #[inline]
    fn flip_bit(&mut self, index: usize) {
        if index >= std::mem::size_of::<Self>() * 8 {
            panic!("index is {index} but length is {}", std::mem::size_of::<Self>() * 8)
        }
        // SAFETY: We checked the index is in bounds
        unsafe { self.flip_bit_unchecked(index) }
    }
}

impl BitGet for [usize] {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        let block_index = index >> WORD_EXP;

        let internal_index = index & WORD_MASK;
        unsafe { self.get_unchecked(block_index).get_bit_unchecked(internal_index) }
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

        unsafe { self.get_unchecked_mut(block_index).set_bit_unchecked(internal_index, value) };
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

        unsafe { self.get_unchecked_mut(block_index).flip_bit_unchecked(internal_index) }
    }

    fn flip_bit(&mut self, index: usize) {
        if index >= self.len() << WORD_EXP {
            panic!("index is {index} but length is {}", self.len() << WORD_EXP)
        }
        unsafe { self.flip_bit_unchecked(index) }
    }
}