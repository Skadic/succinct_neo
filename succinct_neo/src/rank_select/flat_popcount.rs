use crate::bit_vec::BitVec;

use super::traits::RankSupport;

const L1_BLOCK_SIZE: usize = 4096;

const L1_BLOCK_SIZE_EXP: usize = 12;
const L2_BLOCK_SIZE_EXP: usize = 9;

const L2_INDEX_MASK: u128 = (1 << 12) - 1;

// This requires this computer's word size to be 64 bits
static_assertions::assert_eq_size!(usize, u64);

pub struct FlatPopcount<'a> {
    backing: &'a BitVec,
    l1_index: Vec<u128>,
}

impl<'a> FlatPopcount<'a> {
    pub fn new(backing: &'a BitVec) -> Self {
        let n = backing.len();
        let mut temp = Self {
            backing,
            l1_index: Vec::with_capacity((n as f64 / L1_BLOCK_SIZE as f64).ceil() as usize + 1),
        };
        temp.build_l1();
        temp
    }

    fn build_l1(&mut self) {
        let mut num_ones = 0;
        let mut ones_in_l1 = 0;
        let raw_bv = self.backing.raw();

        let mut current_l1 = 0u128;
        for (i, l2_block) in raw_bv.chunks(8).enumerate() {
            let offset = i & 0b0111;
            // In this case this is the last L2 block of this L1 block and we don't store its
            // popcount explicitly
            if offset == 7 {
                // Push the L1 Index entry along with its 7 L2 Index entries to the index
                self.l1_index.push(current_l1);

                // Add the number of ones that occurred in this l1 block
                num_ones += ones_in_l1
                    + l2_block.iter().copied().map(usize::count_ones).sum::<u32>() as usize;
                current_l1 = (num_ones as u128) << 84;
                ones_in_l1 = 0;
                continue;
            }
            // Add the L2 index entry
            ones_in_l1 += l2_block.iter().copied().map(usize::count_ones).sum::<u32>() as usize;
            current_l1 |= (ones_in_l1 as u128 & L2_INDEX_MASK) << (12 * (6 - offset));
        }
        self.l1_index.push(current_l1);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.backing.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn l1(&self, index: usize) -> usize {
        (&self.l1_index[index] >> 84) as usize
    }

    #[inline]
    pub fn l2(&self, l1_index: usize, l2_index: usize) -> usize {
        let offset = 12 * (6 - l2_index);
        ((self.l1_index[l1_index] >> offset) & L2_INDEX_MASK) as usize
    }

    #[inline]
    fn rough_rank_1(&self, l1_index: usize, l2_index: usize) -> usize {
        (if l2_index == 0 {
            let entry = unsafe { *self.l1_index.get_unchecked(l1_index) };
            entry >> 84
        } else {
            let offset_blocks = 7 - l2_index;
            let offset = 12 * offset_blocks;
            let entry = unsafe { *self.l1_index.get_unchecked(l1_index) };
            let l1 = entry >> 84;
            let l2 = (entry >> offset) & L2_INDEX_MASK;
            l1 + l2
        }) as usize
    }
}

impl RankSupport for FlatPopcount<'_> {
    fn rank<const TARGET: bool>(&self, index: usize) -> usize {
        let l1_index = index >> L1_BLOCK_SIZE_EXP;
        let l2_index = (index >> L2_BLOCK_SIZE_EXP) & 0b0111;
        // The index inside of the L2 block
        // Modulus by 512
        let internal_index = index & ((1 << 9) - 1);
        // Number of full words we still need to "popcount"
        // Divide by 64
        let full_remaining_words = internal_index >> 6;
        // remaining number of bits to cover
        let rest_bits = internal_index - (full_remaining_words << 6);

        let mut ones = self.rough_rank_1(l1_index, l2_index);
        let raw_backing = self.backing.raw();
        let word_start = (l1_index << 6) + (l2_index << 3);
        for i in 0..full_remaining_words {
            ones += unsafe { raw_backing.get_unchecked(word_start + i).count_ones() as usize };
        }

        // Add the rest bits
        unsafe {
            ones += (raw_backing.get_unchecked(word_start + full_remaining_words) & ((1 << rest_bits) - 1))
                .count_ones() as usize
        }

        if TARGET {
            ones
        } else {
            index - ones
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{bit_vec::BitVec, rank_select::traits::RankSupport};

    use super::FlatPopcount;

    #[test]
    fn new_test() {
        let mut bv = BitVec::new(10000);

        for i in 0..10000 {
            bv.set(i, i & 2 == 0)
        }

        let pop = FlatPopcount::new(&bv);

        assert_eq!(bv.len(), pop.len(), "length of rank ds not equal to length of bit vec");
        assert!(!pop.is_empty(), "rank ds empty despite not being empty");

        assert_eq!(0, pop.l1(0));
        assert_eq!(2048, pop.l1(1));

        for i1 in 0..2 {
            for i2 in 0..7 {
                assert_eq!(256 * (i2 + 1), pop.l2(i1, i2));
            }
        }
    }

    #[test]
    fn rank_test() {
        let mut bv = BitVec::new(10000);

        for i in 0..10000 {
            bv.set(i, i & 2 == 0)
        }

        let pop = FlatPopcount::new(&bv);

        let mut ones = 0;
        for i in 0..bv.len() {
            assert_eq!(ones, pop.rank::<true>(i), "index {i}");
            assert_eq!(i - ones, pop.rank::<false>(i), "index {i}");
            ones += if bv.get(i) { 1 } else { 0 };
        }
    }
}
