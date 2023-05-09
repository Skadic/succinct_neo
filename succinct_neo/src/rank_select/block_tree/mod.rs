mod construction;
mod pointer;

pub use pointer::PointerBlockTree;

use crate::{
    bit_vec::{
        rank_select::{
            flat_popcount::{BinarySearch, SelectStrategy},
            BitRankSupport, BitSelectSupport, FlatPopcount,
        },
        BitGet, BitVec,
    },
    int_vec::{DynamicIntVec, IntVector},
};

#[allow(unused)]
#[derive(Debug)]
struct BlockTree {
    input_length: usize,
    arity: usize,
    leaf_length: usize,
    /// Maps the characters of the alphabet to a contiguous sequence of characters starting at 0
    mapping: AlphabetMapping,
    /// sizes of a block for each level. index 0 = most shallow level
    level_block_sizes: Vec<usize>,
    /// num blocks for each level
    level_block_count: Vec<usize>,
    is_internal: Vec<FlatPopcount<BitVec, BinarySearch>>,
    /// For every level and every back block, contains the index of the block to which this back
    /// block points. This index ignores back blocks (since back blocks cannot point to other back
    /// blocks)
    back_pointers: Vec<DynamicIntVec>,
    /// For every level and every back block, contains the offset inside its source block from
    /// which to copy
    offsets: Vec<DynamicIntVec>,
    leaf_string: DynamicIntVec,

    // -------------- Rank Information --------------
    // Yeah, we probably don't want this sitting on the stack lol
    /// For each character `c` contains a vector containing an entry for each block on the top
    /// level. This entry contains the number of times `c` appears before the block.
    top_level_block_ranks: [DynamicIntVec; 256],
    // For each char `c` and each level contains an entry for each block containing the number of
    // times the `c` appears inside the block
    block_pop_counts: [Vec<DynamicIntVec>; 256],
    // For each char `c` and each level contains an entry for each *back* block pointing to a source starting in block `b` at offset `i`
    // containing the number of times `c` appears inside of `b` before (exclusively) `i`
    back_block_source_ranks: [Vec<DynamicIntVec>; 256],
}

impl BlockTree {
    pub fn new(
        input: impl AsRef<[u8]>,
        arity: usize,
        leaf_length: usize,
    ) -> Result<Self, &'static str> {
        assert!(arity > 1, "arity must be greater than 1");
        assert!(leaf_length > 0, "leaf length must be greater than 0");

        Ok(PointerBlockTree::new(input.as_ref(), arity, leaf_length)?.into())
    }

    pub fn access(&self, mut i: usize) -> u8 {
        let mut current_level = 0;
        let mut next_level_block_size = self.level_block_sizes[current_level];
        let mut block_idx = 0;
        i -= block_idx * next_level_block_size;

        while current_level < self.level_block_sizes.len() - 1 {
            next_level_block_size = self.level_block_sizes[current_level + 1];
            block_idx = i / next_level_block_size;
            if self.is_internal[current_level].get_bit(block_idx) {
                // What is the index of this block among internal nodes?
                let internal_rank = self.is_internal[current_level].rank::<true>(block_idx);
                // The index at which the children start in the level below this one
                let children_start_index = self.arity * internal_rank;
                let child_index = i / next_level_block_size;
                block_idx = children_start_index + child_index;
                i -= block_idx * next_level_block_size;
                current_level += 1;
            } else {
                // What is the index of this block among back blocks?
                let back_block_rank = self.is_internal[current_level].rank::<false>(block_idx);
                let source = self.back_pointers[current_level].get(back_block_rank);
                i = self.offsets[current_level].get(back_block_rank);
                block_idx = self.is_internal[current_level].select(source).unwrap();
            }
        }
        let leaf_size = next_level_block_size;

        // We should be in a leaf now
        let unmapped_char = self.leaf_string.get(leaf_size * block_idx + i);
        self.mapping.to_ascii(unmapped_char as u8)
    }
}

impl From<PointerBlockTree<'_>> for BlockTree {
    #[inline(always)]
    fn from(value: PointerBlockTree) -> Self {
        Self::construct(value, true)
    }
}

#[derive(Debug, Clone)]
pub struct AlphabetMapping {
    to_ascii: [u8; 256],
    from_ascii: [u8; 256],
}

impl AlphabetMapping {
    pub fn generate(input: &[u8]) -> Self {
        let mut exists = [false; 256];
        exists[0] = true;
        for &c in input.iter() {
            *unsafe { exists.get_unchecked_mut(c as usize) } = true;
        }

        let mut to_ascii = [0u8; 256];
        let mut from_ascii = [0u8; 256];

        for (counter, (character, _)) in exists
            .into_iter()
            .enumerate()
            .filter(|&(_, exists)| exists)
            .enumerate()
        {
            // SAFETY: These counter and character can only be less than 256
            unsafe {
                *from_ascii.get_unchecked_mut(character) = counter as u8;
                *to_ascii.get_unchecked_mut(counter) = character as u8;
            }
        }

        Self {
            to_ascii,
            from_ascii,
        }
    }

    #[inline]
    pub fn to_ascii(&self, code: u8) -> u8 {
        // SAFETY: the array has 256 entries and code is < 256
        unsafe { *self.to_ascii.get_unchecked(code as usize) }
    }

    #[inline]
    pub fn from_ascii(&self, ascii: u8) -> u8 {
        // SAFETY: the array has 256 entries and ascii is < 256
        unsafe { *self.from_ascii.get_unchecked(ascii as usize) }
    }
}

#[cfg(test)]
mod test {
    use super::BlockTree;

    #[test]
    fn new_test() {
        let s = b"verygoodverybaadverygoodverygood";
        //let s = "aaaaaaaaaaaa";
        println!("string len: {}", s.len());
        let bt = BlockTree::new(s, 2, 4).unwrap();

        for (i, c) in s.iter().copied().enumerate() {
            assert_eq!(c, bt.access(i), "incorrect value at index {i}");
        }
    }
}
