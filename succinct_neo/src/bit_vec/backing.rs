use super::{BitGet, BitModify};
use super::{WORD_EXP, WORD_MASK};

macro_rules! primitive_bit_ops {
    {$tp:ty} => {
        impl BitGet for $tp {
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

        impl BitModify for $tp {
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
    };
    {$tp:ty, $($other:ty),+} => {
        primitive_bit_ops!{$tp}
        primitive_bit_ops!{$($other),+}
    }
}

primitive_bit_ops!{
    u8, u16, u32, u64
}

primitive_bit_ops!{ usize }

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

#[cfg(test)]
mod test {
    use crate::bit_vec::{BitGet, BitModify};

    macro_rules! test_primitive {
        {$tp:ty} => {
            paste::paste! {
                #[test]
                pub fn [<$tp _set_get_bit_test>]() {
                    let mut n = 0 as $tp;
                    for i in 0..std::mem::size_of::<$tp>() * 8 {
                        n.set_bit(i, i % 2 == 0);
                    }
                    for i in 0..std::mem::size_of::<$tp>() * 8 {
                        assert_eq!(i % 2 == 0, n.get_bit(i))
                    }
                }

                #[test]
                #[should_panic]
                pub fn [<$tp _get_bit_out_of_bounds_test>]() {
                    let n = 0 as $tp;
                    n.get_bit(std::mem::size_of::<$tp>() * 8);
                }

                #[test]
                #[should_panic]
                pub fn [<$tp _set_bit_out_of_bounds_test>]() {
                    let mut n = 0 as $tp;
                    n.set_bit(std::mem::size_of::<$tp>() * 8, true);
                }

                #[test]
                pub fn [<$tp _flip_bit_test>]() {
                    let mut n = 0 as $tp;
                    for i in 0..std::mem::size_of::<$tp>() * 8 {
                        n.set_bit(i, i % 2 == 0);
                    }
                    for i in 0..std::mem::size_of::<$tp>() * 8 {
                        n.flip_bit(i);
                    }
                    for i in 0..std::mem::size_of::<$tp>() * 8 {
                        assert_eq!(i % 2 == 1, n.get_bit(i), "index {i}")
                    }
                }

                #[test]
                #[should_panic]
                pub fn [<$tp _flip_bit_out_of_bounds_test>]() {
                    let mut n = 0 as $tp;
                    n.flip_bit(std::mem::size_of::<$tp>() * 8);
                }
            }
        };
        {$tp:ty, $($other:ty),+} => {
            test_primitive!{$tp}
            test_primitive!{$($other),+}
        }
    }

    test_primitive!{
        u8, u16, u32, u64, usize
    }

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