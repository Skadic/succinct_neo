use super::L2_INDEX_MASK;

#[cfg(all(
target_arch = "x86_64",
target_feature = "sse2",
target_feature = "ssse3",
target_feature = "sse4.1"
))]
pub use simd::SimdSearch;

pub trait SelectStrategy {
    fn find_l2(entry: u128, rank: usize) -> (usize, usize);
}

/// A search strategy using a simple linear search to locate the correct l2 block.
pub struct LinearSearch;

impl SelectStrategy for LinearSearch {
    fn find_l2(entry: u128, rank: usize) -> (usize, usize) {
        let mut prev = 0;

        for i in 0..7 {
            let l2_entry = ((entry >> (72 - 12 * i)) & L2_INDEX_MASK) as usize;
            if rank < l2_entry {
                return (i, prev);
            }
            prev = l2_entry;
        }

        (7, prev)
    }
}

/// A search strategy using a uniform binary search to locate the correct l2 block.
/// This always requires 3 search steps.
pub struct BinarySearch;

impl SelectStrategy for BinarySearch {
    fn find_l2(entry: u128, rank: usize) -> (usize, usize) {
        macro_rules! l2 {
            ($l2_index:literal) => {
                ((entry >> (72 - 12 * $l2_index)) & L2_INDEX_MASK) as usize
            };
        }

        let l2_3 = l2!(3);
        if rank < l2_3 {
            let l2_1 = l2!(1);
            if rank < l2_1 {
                let l2_0 = l2!(0);
                if rank < l2_0 {
                    (0, 0)
                } else {
                    (1, l2_0)
                }
            } else {
                let l2_2 = l2!(2);
                if rank < l2_2 {
                    (2, l2_1)
                } else {
                    (3, l2_2)
                }
            }
        } else {
            let l2_5 = l2!(5);
            if rank < l2_5 {
                let l2_4 = l2!(4);
                if rank < l2_4 {
                    (4, l2_3)
                } else {
                    (5, l2_4)
                }
            } else {
                let l2_6 = l2!(6);
                if rank < l2_6 {
                    (6, l2_5)
                } else {
                    (7, l2_6)
                }
            }
        }
    }
}

#[cfg(all(
target_arch = "x86_64",
target_feature = "sse2",
target_feature = "ssse3",
target_feature = "sse4.1"
))]
mod simd {
    use super::SelectStrategy;
    use std::arch::x86_64::*;
    use crate::rank_select::flat_popcount::L2_INDEX_MASK;

    pub struct SimdSearch;

    impl SelectStrategy for SimdSearch {
        fn find_l2(mut entry: u128, rank: usize) -> (usize, usize) {
            // We zero the L1 Index data in the entry
            unsafe { *(&mut entry as *mut u128 as *mut u64).offset(1) &= (1 << 20) - 1; }
            let rank = rank as i16;
            let l2_index = unsafe {
                // Put the values into a wide 128 bit register
                let values = _mm_loadu_si128(&entry as *const u128 as *const __m128i);
                // Don't even ask
                let shuffle_mask =
                    _mm_set_epi8(10, 9, 8, 7, 7, 6, 5, 4, 4, 3, 2, 1, 1, 0, -1, -1);
                let values = _mm_shuffle_epi8(values, shuffle_mask);

                // Shift values by 4 bits to the right
                // This is to align the values of odd indices which still need alignment to the
                // byte borders
                let type_2 = _mm_srli_epi16::<4>(values);

                let values = _mm_blend_epi16::<0b0101_0101>(values, type_2);

                // We mask those elements to get rid of the junk we shifted in
                let mask_12_bits = _mm_set1_epi16(0b1111_1111_1111);
                let values = _mm_and_si128(values, mask_12_bits);

                let ranks = _mm_set1_epi16(rank);

                // we get a 128 bit word which contains a 1 as the MSB in each 16 bit block where
                // the l2 index value is greater than the rank we want.
                let result_spread = _mm_cmpgt_epi16(values, ranks);

                // Collect them into a normal integer. Since the last two bytes are empty,
                // we fill the corresponding bits with 1, so our calculations still work when counting leading 0s
                let res = _mm_movemask_epi8(result_spread) | 0b11;

                (res.leading_zeros() - 16) >> 1
            } as usize;
            (l2_index, ((entry >> (84 - 12 * l2_index)) & L2_INDEX_MASK) as usize)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{LinearSearch, BinarySearch, SelectStrategy};

    macro_rules! strat_tests {
        {$strat:ty, $test_name:ident} => {
            paste::paste!{
                #[test]
                fn [<$test_name _1_increment_test>]() {
                    strat_test_1_increment::<$strat>()
                }

                #[test]
                fn [<$test_name _generic_test>]() {
                    strat_test_generic::<$strat>()
                }

                #[test]
                fn [<$test_name _equal_test>]() {
                    strat_test_equal_ranks::<$strat>()
                }
            }
        };
        {$strat:ty, $test_name:ident, $($next_strat:ty, $next_test_name:ident),+} => {
            strat_tests!{$strat, $test_name}
            strat_tests!{$($next_strat, $next_test_name),+}
        }
    }

    strat_tests! {
        LinearSearch, linear_search,
        BinarySearch, binary_search
    }

    #[cfg(all(
    target_arch = "x86_64",
    target_feature = "sse2",
    target_feature = "ssse3",
    target_feature = "sse4.1"
    ))]
    mod simd {
        use super::*;
        use crate::rank_select::flat_popcount::strats::simd::SimdSearch;
        strat_tests! {
            SimdSearch, simd_search
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn strat_test_1_increment<Strat: SelectStrategy>() {
        let mut entry = 0u128;
        // Add random data to the l1 field to ensure this doesn't mess with anything
        entry |= 123456789 << 84;
        entry |= 1;
        entry <<= 12;
        entry |= 2;
        entry <<= 12;
        entry |= 3;
        entry <<= 12;
        entry |= 4;
        entry <<= 12;
        entry |= 5;
        entry <<= 12;
        entry |= 6;
        entry <<= 12;
        entry |= 7;


        for i in 0..128usize {
            assert_eq!((i.min(7), i.min(7)), Strat::find_l2(entry, i), "index {i}");
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn strat_test_generic<Strat: SelectStrategy>() {
        let mut entry = 0u128;
        // Add random data to the l1 field to ensure this doesn't mess with anything
        entry |= 123456789 << 84;
        entry |= 10;
        entry <<= 12;
        entry |= 25;
        entry <<= 12;
        entry |= 80;
        entry <<= 12;
        entry |= 90;
        entry <<= 12;
        entry |= 167;
        entry <<= 12;
        entry |= 1002;
        entry <<= 12;
        entry |= 1762;

        for i in 0..4096usize {
            let expected = match i {
                _ if i < 10 => (0, 0),
                _ if i < 25 => (1, 10),
                _ if i < 80 => (2, 25),
                _ if i < 90 => (3, 80),
                _ if i < 167 => (4, 90),
                _ if i < 1002 => (5, 167),
                _ if i < 1762 => (6, 1002),
                _ => (7, 1762)
            };
            assert_eq!(expected, Strat::find_l2(entry, i), "index {i}");
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn strat_test_equal_ranks<Strat: SelectStrategy>() {
        let mut entry = 0u128;
        // Add random data to the l1 field to ensure this doesn't mess with anything
        entry |= 123456789 << 84;
        entry |= 10;
        entry <<= 12;
        entry |= 25;
        entry <<= 12;
        entry |= 80;
        entry <<= 12;
        entry |= 80;
        entry <<= 12;
        entry |= 167;
        entry <<= 12;
        entry |= 167;
        entry <<= 12;
        entry |= 1762;

        for i in 0..4096usize {
            let expected = match i {
                _ if i < 10 => (0, 0),
                _ if i < 25 => (1, 10),
                _ if i < 80 => (2, 25),
                _ if i < 167 => (4, 80),
                _ if i < 1762 => (6, 167),
                _ => (7, 1762)
            };
            assert_eq!(expected, Strat::find_l2(entry, i), "index {i}");
        }
    }
}
