use crate::{
    bit_vec::BitVec,
    int_vec::{FixedIntVec, IntVector},
    rank_select::block_tree::pointer::{block::Block, Level, PointerBlockTree},
    rolling_hash::{HashedByteMap, HashedByteMultiMap, HashedBytes, RabinKarp, RollingHash},
};

use super::block::{BlockId, BlockType};

impl<'a> PointerBlockTree<'a> {
    /// Calculates the sizes each block should have for each level. Index 0 is the shallowest level (only containing the root).
    ///
    /// # Arguments
    ///
    /// * `input_length`: The length of the input text.
    /// * `arity`: The arity of each tree node.
    /// * `leaf_length`: The number of charactersr in each leaf
    ///
    /// returns: A tuple containing a [Vec] with the block sizes for each level and a [Vec] with the number of blocks for each level.
    ///
    pub(super) fn calculate_level_block_sizes(
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

        (level_block_sizes, level_block_count)
    }

    /// Generate a new level and process it by introducing back pointers
    pub(super) fn process_level(&mut self) -> Result<(), &'static str> {
        self.generate_level().ok_or("could not generate level")?;
        let is_internal = self.scan_block_pairs();
        self.scan_blocks(&is_internal);

        Ok(())
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

        let (current_level, prev_level) = {
            let (current_level, rest) = self.levels.split_last_mut().unwrap();
            let (prev_level, _) = rest.split_last_mut().unwrap();
            (current_level, prev_level)
        };

        let mut current_block_idx = 0;
        let mut last = self.blocks.next_id();
        for &mut prev_block in prev_level {
            if self.blocks[prev_block].is_back_block() {
                continue;
            }
            let prev_start = self.blocks[prev_block].start;
            let prev_len = self.blocks[prev_block].len();
            for i in (0..prev_len).step_by(block_size) {
                // We only want to include blocks which overlap with the input text (this is
                // relevant for the last block of )
                if prev_start + i >= n {
                    break;
                }
                let block = self.blocks.alloc(Block::internal(
                    prev_start + i,
                    prev_start + i + block_size,
                    current_block_idx,
                ));
                current_block_idx += 1;
                current_level.push(block);
                self.blocks[prev_block].add_child(block);
                let next_id = self.blocks.next_id();
                self.blocks[block].set_next(next_id);
                last = block;
            }
        }
        self.blocks[last].clear_next();
        Some(current_level)
    }

    /// Scan through the blocks of the current level pairwise in order to identify leftmost occurrences of block pairs.
    ///
    /// returns: A bit vector where every internal block is marked with a 1 and all back blocks are 0
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
            } else {
                let hashed = rk.hashed_bytes();
                map.entry(hashed).or_insert(hashed);
                rk.advance_n(block_size);
            }
        }

        // Contains an entry for every block
        // Whenever a pair of blocks b_i and b_{i+1} contain the leftmost occurrence of b_i
        // b_{i+1}, the counter for both is incremented
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
                Some(next_next_block) if !next_block.is_adjacent(next_next_block) => 1,
                _ => current_block.len(),
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
            .collect()
    }

    /// Scans through the newest level and saves the hash of every non-internal block in a map.
    /// Then scans through the text with a window of block size.
    /// If the hash of the current window matches the hash of some blocks found in the map,
    /// then create back pointers from those points to the block corresponding to the current window.
    ///
    /// # Arguments
    ///
    /// * `is_internal`: A bit vector that contains a bit for each block. That bit is 1, for every internal block, 0 for each back block.
    /// * `internal_block_first_occurrences`: A map mapping from a block id `i` to the block id,
    /// and offset of its source
    ///
    fn scan_blocks(&mut self, is_internal: &BitVec) {
        let level_depth = self.levels.len() - 1;
        let block_size = self.level_block_sizes[level_depth];
        let num_blocks = self.levels[level_depth].len();

        // Contains the hashes for every block. We save the hash and the block index on this level
        let mut block_hashes = HashedByteMultiMap::<(HashedBytes, usize)>::default();

        // We hash every non-internal blocks and store them in the map
        for i in 0..num_blocks - 1 {
            // Hash the current block
            let rk = RabinKarp::new(
                &self.input[self.block(level_depth, i).unwrap().start..],
                block_size,
            );
            let hashed = rk.hashed_bytes();
            block_hashes.insert(hashed, (hashed, i));
        }

        let mut rk = RabinKarp::new(self.input, block_size);

        for block_index in 0..num_blocks {
            let current_block_id = self.levels[level_depth][block_index];
            let current_block = &self.blocks[current_block_id];
            // The number of times we want to hash inside this block and the start position of the next block
            let (num_hashes, next_block_start, next_adjacent) = {
                let next_block = self.block(level_depth, block_index + 1);
                let next_block_start = next_block.map(|b| b.start);
                let (num_hashes, next_adjacent) = match next_block {
                    Some(next_block) if !current_block.is_adjacent(next_block) => (1, false),
                    _ => (
                        current_block.len()
                            - (current_block.start + current_block.len())
                                .saturating_sub(self.input.len()),
                        true,
                    ),
                };
                (num_hashes, next_block_start, next_adjacent)
            };
            // For each window starting in this block, try to find blocks with the same content
            // If found, set a back pointer
            for offset in 0..num_hashes {
                let hashed = rk.hashed_bytes();
                let current_ptr = hashed.bytes().as_ptr();

                // We search for hashes of blocks with the same hash as the current window
                if let Some(results) = block_hashes.get_vec(&hashed) {
                    for &(block_hash, index) in results {
                        if !is_internal.get(index) {
                            let found_ptr = block_hash.bytes().as_ptr();
                            // SAFETY: We know the pointers are from the same string
                            let byte_offset = unsafe { found_ptr.offset_from(current_ptr) };
                            // This means that `block_hash` is a previous (actually the
                            // leftmost) occurrence of `hashed`
                            if byte_offset > 0 {
                                self.replace(
                                    self.levels[level_depth][index],
                                    current_block_id,
                                    offset,
                                );
                            }
                        } else {
                            // If we find a block that is not to be replaced (yet) we save its
                            // first occurrence and a counter in preparation for the pruning step
                            //internal_block_first_occurrences
                            //    .entry(current_level[index])
                            //    .or_insert((current_level[index], offset));
                            let block_id = self.levels[level_depth][index];
                            let b = &mut self.blocks[block_id];
                            if b.source.is_none() {
                                b.source = Some(block_id);
                                b.offset = Some(offset);
                            }
                        }
                    }
                }
                // We handled this window's content so we remove it from the map
                block_hashes.remove(&hashed);
                rk.advance();
            }
            // This only happens if the next block is not adjacent
            if !next_adjacent {
                // So we recreate the hasher
                rk = RabinKarp::new(&self.input[next_block_start.unwrap()..], block_size);
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn prune(&mut self, block_id: BlockId) {
        // SAFETY: we decouple this block's lifetime from the Arena in order to pass the arena to
        //  the recursive call
        //  This is safe, since a block cannot have itself as a child and the recursive calls only
        //  operate on this block's children
        let me = unsafe { (&mut self.blocks[block_id] as *mut Block).as_mut().unwrap() };
        match me.block_type {
            // If we hit an internal block we prune all children first
            BlockType::Internal {
                ref mut children,
                incident_pointers,
            } => {
                for &child_id in children.iter().rev() {
                    self.prune(child_id);
                }

                // From here, we check requirements that need to hold if we want to replace this
                // block

                // There may not be any other blocks pointing to this
                if incident_pointers != 0 {
                    return;
                }

                // There needs to be a previous occurrence that does not overlap this block
                let (source, offset) = match me.source.zip(me.offset).map(|(block, offset)| {
                    (
                        block,
                        offset,
                        self.blocks[block].start + offset + self.blocks[block].len(),
                    )
                }) {
                    Some((source_id, offset, source_end)) if source_end <= me.start => {
                        (source_id, offset)
                    }
                    _ => return,
                };

                // This is true if all children either are back blocks or have no children (i.e. are
                // leaves)
                if children
                    .iter()
                    .map(|&child_id| &self.blocks[child_id])
                    .any(|child| child.has_children())
                {
                    return;
                }
                // We need to decrement all counters for the blocks the children point to
                for &child_id in children.iter() {
                    // SAFETY: We only modify the blocks the child is pointing to (which
                    // cannot be the child itself)
                    let child =
                        unsafe { (&mut self.blocks[child_id] as *mut Block).as_mut().unwrap() };
                    child.decrement_pointer_count(&mut self.blocks);
                }

                // Now replace this block with a back pointer
                self.replace(block_id, source, offset);
            }
            // If we hit a back block we increment the counters of the blocks it is pointing to
            BlockType::Back { .. } => me.increment_pointer_count(&mut self.blocks),
        };
    }

    #[inline]
    fn replace(&mut self, block_id: BlockId, source: BlockId, offset: usize) {
        let b = &mut self.blocks[block_id];
        b.source = Some(source);
        b.offset = Some(offset);
        b.block_type = BlockType::Back;
    }

    pub(super) fn update_block_indices(&mut self) {
        for level in &self.levels {
            let mut i = 0;
            level
                .iter()
                .for_each(|&block_id| {
                    self.blocks[block_id].index = i;
                    i += 1;
                });
        }
    }
}

#[cfg(test)]
mod test {
    use super::PointerBlockTree;
    use crate::{
        rank_select::block_tree::pointer::{block::BlockType, Level},
        test::res::texts::*,
    };
    use test_case::test_case;

    fn validate_links(bt: &PointerBlockTree, level: &Level) {
        let blocks = level.iter().map(|&id| &bt.blocks[id]).collect::<Vec<_>>();
        for block in blocks {
            if let BlockType::Back { .. } = block.block_type {
                let src_id = unsafe { block.source.unwrap_unchecked() };
                let offset = unsafe { block.offset.unwrap_unchecked() };
                let source_block = &bt.blocks[src_id];
                let source_start = source_block.start + offset;
                let len = block.end.min(bt.input_length()) - block.start;
                assert_ne!(block, source_block, "cannot link block to itself");
                assert_eq!(
                    &bt.input[block.start..block.start + len],
                    &bt.input[source_start..source_start + len],
                    "invalid pointer for block at index {}",
                    block.start
                )
            }
        }
    }

    #[test_case(ALL_A; "all_a")]
    #[test_case(DNA; "dna")]
    #[test_case(EINSTEIN; "einstein")]
    fn valid_back_pointers_test(input: &'static str) {
        let bt = PointerBlockTree::new(input.as_bytes(), 4, 8).unwrap();
        for level in bt.levels.iter() {
            validate_links(&bt, level);
        }
    }
}
