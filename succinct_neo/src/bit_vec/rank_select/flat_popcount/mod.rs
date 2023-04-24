use crate::bit_vec::{BitGet, BitVec};
use std::borrow::Borrow;
use std::marker::PhantomData;

use crate::bit_vec::rank_select::traits::{BitRankSupport, BitSelectSupport};
use crate::int_vec::{DynamicIntVec, IntVector};

/// The number of bits in an L1 block
const L1_BLOCK_SIZE: usize = 4096;
/// The number of bits in an L2 block
const L2_BLOCK_SIZE: usize = 512;

/// $2^12 = 4096$, the L1 block size
const L1_BLOCK_SIZE_EXP: usize = L1_BLOCK_SIZE.ilog2() as usize;
/// $2^12 = 512$, the L2 block size
const L2_BLOCK_SIZE_EXP: usize = L2_BLOCK_SIZE.ilog2() as usize;

/// The mask covering the size of an L2 index entry (12 bits)
const L2_INDEX_MASK: u128 = (1 << L1_BLOCK_SIZE_EXP) - 1;

// This requires this computer's word size to be 64 bits
static_assertions::assert_eq_size!(usize, u64);

mod strats;

pub use strats::*;

/// An implementation of the rank/select data structure described by Florian Kurpicz in his paper
/// *Engineering Compact Data Structures for Rank and Select Queries on Bit Vectors*.
/// The paper can be found [here](https://arxiv.org/abs/2206.01149).
///
/// This data structure should work well in most cases with a low memory overhead over the
/// bitvector (less than 4%).
pub struct FlatPopcount<Backing, Strat = LinearSearch>
where
    Backing: Borrow<BitVec>,
{
    backing: Backing,
    l1_index: Vec<u128>,
    sampled_ones: DynamicIntVec,
    number_of_ones: usize,
    _strat_mark: PhantomData<Strat>,
}

impl<Strat, Backing> FlatPopcount<Backing, Strat>
where
    Backing: Borrow<BitVec>,
{
    /// Creates a new rank data structure from a bit vector.
    ///
    /// # Arguments
    ///
    /// * `backing` - The backing bitvector
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{FlatPopcount, BitRankSupport}
    /// };
    ///
    /// let mut bv = BitVec::new(64);
    ///
    /// bv.flip(10);
    /// bv.flip(15);
    /// bv.flip(20);
    ///
    /// let rank_ds = FlatPopcount::<()>::new(&bv);
    /// assert_eq!(2, rank_ds.rank::<true>(17));
    /// assert_eq!(12, rank_ds.rank::<false>(13));
    /// ```
    pub fn new(backing: Backing) -> Self {
        let n = backing.borrow().len();
        if n == 0 {
            return Self {
                backing,
                l1_index: Vec::with_capacity(0),
                sampled_ones: DynamicIntVec::new(1),
                _strat_mark: Default::default(),
                number_of_ones: 0,
            };
        }

        let log_n = n.ilog2() as usize + 1;
        let mut temp = Self {
            backing,
            l1_index: Vec::with_capacity((n as f64 / L1_BLOCK_SIZE as f64).ceil() as usize + 1),
            sampled_ones: DynamicIntVec::new(log_n),
            _strat_mark: Default::default(),
            number_of_ones: 0,
        };
        temp.build_indices();
        temp.sample_ones();
        temp
    }

    /// Builds the required backing index data structure.
    fn build_indices(&mut self) {
        let mut num_ones = 0;
        let mut ones_in_l1 = 0;
        let raw_bv = self.backing.borrow().raw();

        let mut current_l1 = 0u128;
        let mut i = 0;
        for l2_block in raw_bv.chunks(8) {
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
                i += 1;
                continue;
            }
            // Add the L2 index entry
            ones_in_l1 += l2_block.iter().copied().map(usize::count_ones).sum::<u32>() as usize;
            current_l1 |= (ones_in_l1 as u128 & L2_INDEX_MASK) << (12 * (6 - offset));
            i += 1;
        }
        // Fill the unused L2 blocks in the last L1 Block with all ones
        while i & 0b0111 != 7 {
            current_l1 |= L2_INDEX_MASK << (12 * (6 - (i & 0b0111)));
            i += 1;
        }
        self.l1_index.push(current_l1);
    }

    /// Samples every 8192nd one and saves the l1 block it is in
    fn sample_ones(&mut self) {
        let mut count = -1isize;
        for (i, value) in self.backing.borrow().iter().enumerate() {
            if value {
                count += 1;
                if count & ((1 << 13) - 1) == 0 {
                    self.sampled_ones.push(i >> 13);
                }
            }
        }
        self.number_of_ones = (count + 1) as usize;
    }

    /// Gets the number of bits in the underlying bit vector.
    ///
    /// This is *not* the number of ones in the bit vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{FlatPopcount, BitRankSupport}
    /// };
    ///
    /// let bv = BitVec::new(64);
    /// let rank_ds = FlatPopcount::<()>::new(&bv);
    /// assert_eq!(bv.len(), rank_ds.len());
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.backing.borrow().len()
    }

    /// Returns `true`, if the backing bit vector is empty.
    ///
    /// That is, this returns `true` of there is no space for any bits in the underlying bitvector.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{FlatPopcount, BitRankSupport}
    /// };
    ///
    /// let bv = BitVec::new(64);
    /// let rank_ds = FlatPopcount::<()>::new(&bv);
    /// assert!(!rank_ds.is_empty());
    ///
    /// let bv = BitVec::new(0);
    /// let rank_ds = FlatPopcount::<()>::new(&bv);
    /// assert!(rank_ds.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of ones in the entire bitvector.
    pub fn num_ones(&self) -> usize {
        self.number_of_ones
    }

    /// Calculates the number of ones up to and not including the given l2 block.
    ///
    /// # Arguments
    ///
    /// * `l1_index` - The index of the L1 Block
    /// * `l2_index` - The index of the L2 Block inside of the L1 Block (valid range is 0-7
    /// inclusively)
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

    #[inline]
    unsafe fn l1(&self, l1_index: usize) -> usize {
        *((self.l1_index.get_unchecked(l1_index) as *const u128 as *const usize).offset(1)) >> 20
    }

    #[inline]
    /// Find the l1 index entry containing the 1 with the given rank
    ///
    /// SAFETY:
    ///
    /// The l1 start index must be in range of the l1 index.
    unsafe fn find_l1(&self, l1_start_index: usize, rank: usize) -> usize {
        let n = self.l1_index.len();
        let mut ptr =
            (self.l1_index.get_unchecked(l1_start_index) as *const u128 as *const usize).add(1);
        // Find the l1 block that contains the 1 we need
        for l1_index in l1_start_index..n {
            let l1 = *ptr >> 20;
            if l1 > rank {
                return l1_index - 1;
            }
            ptr = ptr.add(2);
        }
        n - 1
    }
}
impl<Strat, Backing> BitRankSupport for FlatPopcount<Backing, Strat> where Backing: Borrow<BitVec> {
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
        let raw_backing = self.backing.borrow().raw();
        let word_start = (l1_index << 6) + (l2_index << 3);
        for i in 0..full_remaining_words {
            ones += unsafe { raw_backing.get_unchecked(word_start + i).count_ones() as usize };
        }

        if rest_bits > 0 {
            // Add the rest bits
            unsafe {
                const WORD_SIZE: usize = std::mem::size_of::<usize>() * 8;
                let word = *raw_backing.get_unchecked(word_start + full_remaining_words);
                let mask = ((1usize << rest_bits) - 1) << (WORD_SIZE - rest_bits);
                ones += (word & mask).count_ones() as usize
            }
        }

        if TARGET {
            ones
        } else {
            index - ones
        }
    }
}

impl<Strat: SelectStrategy, Backing> BitSelectSupport<true> for FlatPopcount<Backing, Strat> where Backing: Borrow<BitVec> {
    fn select(&self, mut rank: usize) -> Option<usize> {
        if rank >= self.number_of_ones {
            return None;
        }
        let l1_index = self.sampled_ones.get(rank >> 13);
        // SAFETY: The data in sampled_ones should be correct, so this must work too
        let l1_index = unsafe { self.find_l1(l1_index, rank) };
        rank -= unsafe { self.l1(l1_index) };

        // Find the correct l2 block inside the l1 block
        let block = unsafe { *self.l1_index.get_unchecked(l1_index) };
        let (l2_index, ones_in_l2) = Strat::find_l2(block, rank);
        rank -= ones_in_l2;

        // Find the correct word inside the l2 block
        let mut current_index = (l1_index << 6) + (l2_index << 3);
        let mut index_in_l2 = 0;
        loop {
            let num_ones =
                unsafe { self.backing.borrow().raw().get_unchecked(current_index).count_ones() as usize };
            if num_ones <= rank {
                rank -= num_ones;
                current_index += 1;
                index_in_l2 += 1;
            } else {
                break;
            }
        }

        // Find the correct 1 inside the word
        let word = unsafe { *self.backing.borrow().raw().get_unchecked(current_index) };
        let mut index_in_word = 0;
        loop {
            let bit = unsafe { word.get_bit_unchecked(index_in_word) };
            if rank == 0 && bit {
                break;
            }
            // SAFETY: indices are <= 64
            if bit {
                rank -= 1;
            }
            index_in_word += 1;
        }

        Some(
            (l1_index << L1_BLOCK_SIZE_EXP)
                + (l2_index << L2_BLOCK_SIZE_EXP)
                + (index_in_l2 << 6)
                + index_in_word,
        )
    }
}

impl<T, Backing: Borrow<BitVec>> BitGet for FlatPopcount<Backing, T> {
    #[inline]
    unsafe fn get_bit_unchecked(&self, index: usize) -> bool {
        self.backing.borrow().get_bit_unchecked(index)
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        self.backing.borrow().get_bit(index)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Borrow;

    use super::{FlatPopcount, L2_INDEX_MASK};
    use crate::{
        bit_vec::{
            rank_select::{
                flat_popcount::BinarySearch,
                traits::{BitRankSupport, BitSelectSupport},
            },
            BitVec,
        },
        int_vec::IntVector,
    };

    #[inline]
    fn l1<T>(pop: &FlatPopcount<impl Borrow<BitVec>, T>, index: usize) -> usize {
        (&pop.l1_index[index] >> 84) as usize
    }

    #[inline]
    fn l2<T>(pop: &FlatPopcount<impl Borrow<BitVec>, T>, l1_index: usize, l2_index: usize) -> usize {
        let offset = 12 * (6 - l2_index);
        ((pop.l1_index[l1_index] >> offset) & L2_INDEX_MASK) as usize
    }

    #[test]
    fn new_test() {
        let mut bv = BitVec::new(50000);

        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }

        let pop = FlatPopcount::<_, ()>::new(&bv);

        assert_eq!(
            bv.len(),
            pop.len(),
            "length of rank ds not equal to length of bit vec"
        );
        assert!(!pop.is_empty(), "rank ds empty despite not being empty");

        assert_eq!(0, l1(&pop, 0));
        assert_eq!(2048, l1(&pop, 1));

        for i1 in 0..bv.len() / 4096 {
            for i2 in 0..7 {
                assert_eq!(
                    256 * (i2 + 1),
                    l2(&pop, i1, i2),
                    "l2 entry {i2} in l1 entry {i1}"
                );
            }
        }

        for i in 0..bv.len() / 16384 {
            assert_eq!(
                (i * 16384) / 8192,
                pop.sampled_ones.get(i),
                "sampled position for {i}'th one"
            );
        }
    }

    #[test]
    fn rank_test() {
        let mut bv = BitVec::new(10000);

        for i in 0..bv.len() {
            bv.set(i, i & 2 == 0)
        }

        let pop = FlatPopcount::<_, ()>::new(&bv);

        let mut ones = 0;
        for i in 0..bv.len() {
            assert_eq!(ones, pop.rank::<true>(i), "index {i}");
            assert_eq!(i - ones, pop.rank::<false>(i), "index {i}");
            ones += if bv.get(i) { 1 } else { 0 };
        }
    }

    #[test]
    fn select_test() {
        let mut bv = BitVec::new(50000);

        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }

        let pop = FlatPopcount::<_, BinarySearch>::new(&bv);
        for i in 1..bv.len() / 2 {
            assert_eq!(
                Some(2 * i),
                pop.select(i),
                "{i}th one should be at index {}",
                2 * i
            );
        }
        assert_eq!(
            None,
            pop.select(bv.len() / 2),
            "should return None if rank is higher than number of ones"
        );
    }

    #[test]
    fn select_exceed_test() {
        let mut bv = BitVec::new(50000);

        for i in 0..bv.len() {
            bv.set(i, i % 2 == 0)
        }

        let pop = FlatPopcount::<_, BinarySearch>::new(&bv);

        assert_eq!(None, pop.select(100000));
    }
}
