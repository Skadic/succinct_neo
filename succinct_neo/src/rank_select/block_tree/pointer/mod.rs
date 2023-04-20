use self::block::{Block, BlockId};
use id_arena::Arena;

mod block;
mod construction;

type Level = Vec<BlockId>;

pub struct PointerBlockTree<'a> {
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

impl<'a> PointerBlockTree<'a> {
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

        // We process each level of the tree
        while bt.process_level().is_ok() {}

        Ok(bt)
    }

    #[inline]
    pub fn input_length(&self) -> usize {
        self.input.len()
    }

    fn block(&self, level: usize, idx: usize) -> Option<&Block> {
        let id = *self.levels.get(level).and_then(|level| level.get(idx))?;
        self.blocks.get(id)
    }

    fn root(&self) -> &Block {
        &self.blocks[self.root]
    }

    pub fn get(&self, i: usize) -> u8 {
        self.root().get(self, i)
    }
}

#[cfg(test)]
mod test {
    use crate::test::res::texts::*;
    use test_case::test_case;

    use super::PointerBlockTree;

    #[test_case(ALL_A; "all_a")]
    #[test_case(DNA; "dna")]
    #[test_case(EINSTEIN; "einstein")]
    fn get_test(input: &'static str) {
        let input = input.as_bytes();
        let bt = PointerBlockTree::new(input, 4, 8).unwrap();
        for (i, &c) in input.iter().enumerate() {
            assert_eq!(c, bt.get(i), "mismatch as index {i}");
        }
    }
}
