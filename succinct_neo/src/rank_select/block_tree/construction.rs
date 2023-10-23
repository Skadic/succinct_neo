use std::mem::MaybeUninit;

use itertools::Itertools;

use crate::{
    bit_vec::{
        rank_select::{BitRankSupport, FlatPopcount},
        BitVec,
    },
    int_vec::{DynamicIntVec, IntVector},
};

use super::{pointer::block::Block, AlphabetMapping, BlockTree, PointerBlockTree};

fn num_bits(v: usize) -> usize {
    ((v + 1) as f64).log2().ceil() as usize
}

impl BlockTree {
    pub(super) fn construct(pbt: PointerBlockTree, rank: bool) -> Self {
        let mapping = AlphabetMapping::generate(pbt.input);
        let top_level_index = Self::top_level_index(&pbt);

        let mut bt = Self {
            input_length: pbt.input_length(),
            arity: pbt.arity,
            leaf_length: pbt.leaf_length,
            mapping,
            level_block_sizes: pbt
                .level_block_sizes
                .iter()
                .copied()
                .skip(top_level_index)
                .collect(),
            level_block_count: pbt
                .level_block_count
                .iter()
                .copied()
                .skip(top_level_index)
                .collect(),
            is_internal: Vec::new(),
            back_pointers: Vec::new(),
            offsets: Vec::new(),
            top_level_block_ranks: fill_arr(DynamicIntVec::with_capacity(1, 0)),
            block_pop_counts: fill_arr(Vec::with_capacity(0)),
            back_block_source_ranks: fill_arr(Vec::with_capacity(0)),
            leaf_string: DynamicIntVec::with_capacity(1, 0),
        };
        println!("{bt:?}");

        // Prepare top level ranks
        if rank {
            bt.calculate_top_level_ranks(pbt.input);
        }

        for level_index in top_level_index..pbt.levels.len() {
            bt.process_pbt_level(&pbt, level_index)
        }

        bt
    }

    /// We "cut off" levels of the original block tree as long as there aren't any back blocks on
    /// them. This calculates the index of the first level with back blocks.
    fn top_level_index(pbt: &PointerBlockTree) -> usize {
        let mut level_depth = 0;
        while level_depth < pbt.levels.len()
            && pbt.levels[level_depth]
                .iter()
                .map(|&block_id| &pbt.blocks[block_id])
                .all(|block| block.is_internal())
        {
            level_depth += 1;
        }

        level_depth.min(pbt.levels.len() - 1)
    }

    fn process_pbt_level(&mut self, pbt: &PointerBlockTree, level_index: usize) {
        let level = pbt.levels[level_index]
            .iter()
            .map(|&id| &pbt.blocks[id])
            .collect_vec();

        // If this is the last level we need the leaf string
        if level_index == pbt.levels.len() - 1 {
            println!("ok this actually happens");
            self.leaf_string = level
                .iter()
                .flat_map(|block| pbt.input[block.start..block.end].iter().copied())
                .map(|byte| self.mapping.from_ascii(byte))
                .collect();
        }

        // Bit Vector depicting whether each block is internal
        let is_internal = FlatPopcount::<_, _>::new(
            level
                .iter()
                .copied()
                .map(Block::is_internal)
                .collect::<BitVec>(),
        );

        let back_pointers = level
            .iter()
            .filter(|b| b.is_back_block())
            .map(|b| b.source.map(|id| &pbt.blocks[id].index).unwrap())
            .map(|&index| is_internal.rank::<false>(index))
            .collect::<DynamicIntVec>();

        let offsets = level
            .iter()
            .filter(|b| b.is_back_block())
            .map(|b| b.offset.unwrap())
            .collect::<DynamicIntVec>();

        self.is_internal.push(is_internal);
        self.back_pointers.push(back_pointers);
        self.offsets.push(offsets);
    }

    fn calculate_top_level_ranks(&mut self, input: &[u8]) {
        // Count the characters and allocate memory to fit the number of characters
        let mut char_counts = [0; 256];
        for c in input.iter().map(|&c| c as usize) {
            // SAFETY: chars are always < 256
            unsafe {
                *char_counts.get_unchecked_mut(c) += 1;
            }
        }
        let top_level_block_count = self.level_block_count[0];
        for (i, &count) in char_counts
            .iter()
            .enumerate()
            .filter(|&(_, &count)| count > 0)
        {
            self.top_level_block_ranks[i] =
                DynamicIntVec::with_capacity(count, top_level_block_count);
        }

        let mut new_char_counts = [0; 256];
        let mut input_iter = input.iter().copied();
        let block_size = self.level_block_sizes[0];
        // for every block count the characters (cumulatively) and save them to the
        // top_level_block_ranks field
        for _ in 0..*self.level_block_count.last().unwrap() {
            for (c, &count) in new_char_counts
                .iter()
                .enumerate()
                .filter(|&(i, _)| char_counts[i] > 0)
            {
                self.top_level_block_ranks[c].push(count);
            }
            for _ in 0..block_size {
                let Some(c) = input_iter.next() else {
                    break;
                };
                // SAFETY: chars are always < 256
                unsafe {
                    *new_char_counts.get_unchecked_mut(c as usize) += 1;
                }
            }
        }
    }
}

fn fill_arr<T: Clone, const N: usize>(v: T) -> [T; N] {
    // SAFETY: We know we will fill this momentarily
    unsafe {
        let mut s = MaybeUninit::<[T; N]>::uninit();
        for i in 0..N {
            (*s.as_mut_ptr()).as_mut_ptr().add(i).write(v.clone())
        }
        s.assume_init()
    }
}
