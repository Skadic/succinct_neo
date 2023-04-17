use self::block::{Block, BlockId};
use id_arena::Arena;

mod block;
mod construction;

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

        while bt.process_level().is_ok() {}

        println!("{:#?}", bt.blocks);
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

    fn block_mut(&mut self, level: usize, idx: usize) -> Option<&mut Block> {
        let id = *self.levels.get(level).and_then(|level| level.get(idx))?;
        self.blocks.get_mut(id)
    }
}
