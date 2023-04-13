use std::collections::hash_map::Entry;

use id_arena::Arena;

use crate::{
    bit_vec::BitVec,
    int_vec::{FixedIntVec, IntVector},
    rolling_hash::{HashedByteMap, RabinKarp, RollingHash},
};

use self::block::{Block, BlockId};

mod block;

type Level = Vec<BlockId>;

pub(crate) struct IntermediateBlockTree<'a> {
    /// Arena for allocating the blocks to hopefully have some semblance of cache efficiency
    blocks: Arena<Block>,
    root: BlockId,
    input: &'a [u8],
    arity: usize,
    leaf_length: usize,
    /// sizes of a block for each level. index 0 = most shallow level
    level_block_sizes: Vec<usize>,
    /// num blocks for each level
    level_block_count: Vec<usize>,
    levels: Vec<Level>,
}

impl<'a> IntermediateBlockTree<'a> {
    pub fn new(input: &'a [u8], arity: usize, leaf_length: usize) -> Result<Self, &'static str> {
        assert!(arity > 1, "arity must be greater than 1");
        assert!(leaf_length > 0, "leaf length must be greater than 0");
        let mut blocks = Arena::new();
        let (level_block_sizes, level_block_count) =
            Self::calculate_level_block_sizes(input.len(), arity, leaf_length);
        // We allocate the root block
        let root = blocks.alloc(Block::internal(0, level_block_sizes[0]));

        let mut bt = Self {
            blocks,
            input,
            root,
            arity,
            levels: vec![vec![root]],
            leaf_length,
            level_block_sizes,
            level_block_count,
        };

        bt.process_level()?;
        bt.process_level()?;
        println!("{:?}", bt.blocks);
        Ok(bt)
    }

    #[inline]
    pub fn input_length(&self) -> usize {
        self.input.len()
    }

    fn calculate_level_block_sizes(
        input_length: usize,
        arity: usize,
        leaf_length: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let num_levels = (input_length as f64 / leaf_length as f64)
            .log(arity as f64)
            .ceil() as usize;
        let mut level_block_sizes = Vec::with_capacity(num_levels);
        let mut level_block_count = Vec::with_capacity(num_levels);
        let float_length = input_length as f64;

        let mut block_size = leaf_length;

        while block_size < input_length {
            level_block_sizes.push(block_size);
            level_block_count.push((float_length / block_size as f64).ceil() as usize);
            block_size *= arity;
        }

        level_block_sizes.push(block_size);
        level_block_count.push(1);

        level_block_sizes.reverse();
        level_block_count.reverse();

        println!("sizes: {:?}", level_block_sizes);
        println!("count: {:?}", level_block_count);

        (level_block_sizes, level_block_count)
    }

    /// Generates a new level. Returns a mutable reference to the level if there actually was a level to be generated, `None`
    /// otherwise.
    fn generate_level(&mut self) -> Option<&mut Level> {
        let level_depth = self.levels.len();
        if level_depth >= self.level_block_sizes.len() {
            return None;
        }
        let block_size = self.level_block_sizes[level_depth];
        let num_blocks = self.level_block_count[level_depth];
        let n = self.input_length();
        // Insert a new vector to hold this level
        self.levels.push(Vec::with_capacity(num_blocks));
        let (v, prev_level) = {
            let (v, rest) = self.levels.split_last_mut().unwrap();
            let (prev_level, _) = rest.split_last_mut().unwrap();
            (v, prev_level)
        };
        for &mut prev_block in prev_level {
            let prev_start = self.blocks[prev_block].start;
            let prev_len = self.blocks[prev_block].len();
            for i in (0..prev_len).step_by(block_size) {
                // We only want to include blocks which overlap with the input text (this is
                // relevant for the last block of )
                if prev_start + i >= n {
                    break;
                }
                let block = self
                    .blocks
                    .alloc(Block::internal(prev_start + i, prev_start + i + block_size));
                v.push(block);
                self.blocks[prev_block].add_child(block);
            }
        }
        Some(v)
    }

    fn process_level(&mut self) -> Result<(), &'static str> {
        self.generate_level().ok_or("could not generate level")?;
        let marked_pairs = self.scan_block_pairs();
        println!("{marked_pairs:?}");

        Ok(())
    }

    fn block(&self, level: usize, idx: usize) -> Option<&Block> {
        let id = *self.levels.get(level).and_then(|level| level.get(idx))?;
        self.blocks.get(id)
    }

    fn block_mut(&mut self, level: usize, idx: usize) -> Option<&mut Block> {
        let id = *self.levels.get(level).and_then(|level| level.get(idx))?;
        self.blocks.get_mut(id)
    }

    /// Scan through the blocks of the current level pairwise in order to identify leftmost occurrences of block pairs.
    ///
    /// # Arguments
    ///
    /// * `s` - The input string
    ///
    /// returns: A bit vector where every marked pair of blocks is marked with a 1
    fn scan_block_pairs(&mut self) -> BitVec {
        let level_depth = self.levels.len() - 1;
        let block_size = self.level_block_sizes[level_depth];
        let num_blocks = self.levels[level_depth].len();
        let pair_size = 2 * block_size;

        let mut rk = RabinKarp::new(self.input, pair_size);

        // Contains the hashes for every pair of blocks
        let mut map = HashedByteMap::default();

        // We hash every pair of blocks and store them in the map
        for i in 0..num_blocks - 1 {
            let current_block = self.block(level_depth, i).unwrap();
            let next_block = self.block(level_depth, i + 1).unwrap();
            if !current_block.is_adjacent(next_block) {
                // Skip non-adjacent blocks
                rk = RabinKarp::new(&self.input[next_block.start..], pair_size);
                continue;
            }
            let hashed = rk.hashed_bytes();
            map.entry(hashed).or_insert(hashed);
            rk.advance_n(block_size);
        }

        // Contains an entry for every block pair
        // b_i b_{i+1} is marked <=> b_i b{i+1} are the leftmost occurrence of their
        // respective substring
        // We start by considering every pair as being the leftmost occurrence
        // when we find an occurrence further to the left, we un-mark the pair
        let mut pair_marks = FixedIntVec::<2>::with_capacity(num_blocks);
        (0..num_blocks).for_each(|_| pair_marks.push(0));

        let mut rk = RabinKarp::new(self.input, pair_size);
        for block_index in 0..num_blocks - 1 {
            let current_block = self.block(level_depth, block_index).unwrap();
            let next_block = self.block(level_depth, block_index + 1).unwrap();
            if !current_block.is_adjacent(next_block) {
                rk = RabinKarp::new(&self.input[next_block.start..], pair_size);
                continue;
            }

            // The number of times the hasher should advance inside the current block,
            // If the next block and the one after that are not adjacent, then we may only hash
            // once (=exactly the current and the next block)
            let num_hashes = match self.block(level_depth, block_index + 2) {
                Some(next_next_block) if next_block.is_adjacent(next_next_block) => current_block.len(),
                Some(_) => 1,
                None => current_block.len(),
            };

            for _ in 0..num_hashes {
                let hashed = rk.hashed_bytes();
                let ptr = hashed.bytes().as_ptr();

                match map.get(&hashed) {
                    None => {}
                    // hash of some block pair that has the same hash as the current window
                    Some(&pair_hash) => {
                        let found_ptr = pair_hash.bytes().as_ptr();
                        // If the pointers are equal this means that the hash we found is of the block
                        // itself (i.e. the block is the leftmost occurence)
                        if ptr == found_ptr {
                            pair_marks.set(block_index, pair_marks.get(block_index) + 1);
                            pair_marks.set(block_index + 1, pair_marks.get(block_index + 1) + 1);
                            map.remove(&pair_hash);
                        }
                    }
                }
                rk.advance();
            }
        }

        pair_marks
            .into_iter()
            .enumerate()
            .map(|(i, v)| v == 2 || i == 0 || i == num_blocks - 1 && v == 1)
            .collect::<BitVec>()
    }


}

#[cfg(test)]
mod test {
    use super::IntermediateBlockTree;

    #[test]
    fn block_size_test() {
        // Char with 48 characters
        let s = std::iter::repeat("abcdef".chars())
            .take(7)
            .flatten()
            .chain("abcdeg".chars())
            .collect::<String>();
        println!("input len: {}", s.len());
        let bt = IntermediateBlockTree::new(s.as_bytes(), 3, 2);
    }
}
