use std::collections::hash_map::Entry;

use crate::{
    bit_vec::BitVec,
    rolling_hash::{HashedByteMap, RabinKarp, RollingHash},
};

mod intermediate;

#[derive(Debug)]
#[allow(unused)]
struct BlockTree {
    input_length: usize,
    arity: usize,
    leaf_length: usize,
    /// sizes of a block for each level. index 0 = most shallow level
    level_block_sizes: Vec<usize>,
    /// num blocks for each level
    level_block_count: Vec<usize>,
}

impl BlockTree {
    pub fn new(input: impl AsRef<[u8]>, arity: usize, leaf_length: usize) -> Self {
        assert!(arity > 1, "arity must be greater than 1");
        assert!(leaf_length > 0, "leaf length must be greater than 0");
        let s = input.as_ref();

        let mut bt = BlockTree {
            input_length: s.len(),
            arity,
            leaf_length,
            level_block_sizes: Vec::with_capacity(0),
            level_block_count: Vec::with_capacity(0),
        };

        bt.calculate_level_block_sizes();
        bt.scan_block_pairs(s, 2);

        bt
    }

    fn calculate_level_block_sizes(&mut self) {
        let num_levels = (self.input_length as f64 / self.leaf_length as f64)
            .log(self.arity as f64)
            .ceil() as usize;
        self.level_block_sizes = Vec::with_capacity(num_levels);
        self.level_block_count = Vec::with_capacity(num_levels);
        let float_length = self.input_length as f64;

        let mut block_size = self.leaf_length;

        while block_size < self.input_length {
            self.level_block_sizes.push(block_size);
            self.level_block_count
                .push((float_length / block_size as f64).ceil() as usize);
            block_size *= self.arity;
        }

        self.level_block_sizes.reverse();
        self.level_block_count.reverse();

        println!("sizes: {:?}", self.level_block_sizes);
        println!("count: {:?}", self.level_block_count);
    }

    fn process_level(&mut self, s: &[u8], level: usize) {
        let marked_pairs = self.scan_block_pairs(s, level);
    }

    /// Scan through the blocks pairwise in order to identify leftmost occurrences of block pairs.
    ///
    /// # Arguments
    ///
    /// * `s` - The input string
    /// * `level` - The current level
    ///
    /// returns: A bit vector where every marked pair of blocks is marked with a 1
    fn scan_block_pairs<'a>(&mut self, s: &'a [u8], level: usize) -> BitVec {
        let block_size = self.level_block_sizes[level];
        let num_blocks = self.level_block_count[level];
        let pair_size = 2 * block_size;

        dbg!(block_size, num_blocks, pair_size);

        let mut rk = RabinKarp::new(s, pair_size);

        // Contains the hashes for every pair of blocks
        let mut map = HashedByteMap::default();

        // Contains an entry for every block pair
        // b_i b_{i+1} is marked <=> b_i b{i+1} are the leftmost occurrence of their
        // respective substring
        // We start by considering every pair as being the leftmost occurrence
        // when we find an occurrence further to the left, we un-mark the pair
        let mut pair_marks = BitVec::one(num_blocks);

        // We hash every pair of blocks and store them in the map
        for i in 0..num_blocks - 1 {
            let hashed = rk.hashed_bytes();
            match map.entry(hashed) {
                Entry::Occupied(_) => pair_marks.set(i, false),
                Entry::Vacant(e) => {
                    e.insert(hashed);
                }
            };
            rk.advance_n(block_size);
        }

        let mut rk = RabinKarp::new(s, pair_size);
        for _ in 0..s.len() {
            let hashed = rk.hashed_bytes();
            let ptr = hashed.bytes().as_ptr();

            match map.get(&hashed) {
                None => {}
                // hash of some block pair that has the same hash as the current window
                Some(pair_hash) => {
                    println!("Found: {pair_hash:?}");
                    let found_ptr = pair_hash.bytes().as_ptr();
                    // SAFTEY: We know the pointers are both from the text s
                    let offset = unsafe { found_ptr.offset_from(ptr) };
                    // This means that the hash we found is of some block pair later than where we
                    // are now
                    println!("Offset: {offset}");
                    if offset > 0 && hashed.bytes() == pair_hash.bytes() {
                        // We must find the index of the found pair
                        // Remember that the offset is in bytes and each block is `block_size`
                        // bytes wide
                        let block_idx = offset as usize / block_size;
                        pair_marks.set(block_idx, false);
                        // TODO We could insert our hashed pair into the map here, to save the
                        // leftmost occurrence
                    }
                }
            }
            rk.advance();
        }

        // calculate the marking of single blocks from
        let mut prev = true;
        for i in 1..num_blocks - 1 {
            let marked = prev || pair_marks.get(i);
            prev = pair_marks.get(i);
            pair_marks.set(i - 1, marked);
        }

        pair_marks
    }
}
#[cfg(test)]
mod test {
    use super::BlockTree;

    #[test]
    fn new_test() {
        let s = "verygoodverybaadverygoodverygood";
        //let s = "aaaaaaaaaaaa";
        println!("string len: {}", s.len());
        let bt = BlockTree::new(s, 2, 4);
    }
}
