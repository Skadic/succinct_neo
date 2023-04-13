#![allow(unused)]

use id_arena::Id;

pub(crate) type BlockId = Id<Block>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BlockType {
    Internal(Vec<BlockId>),
    Back(BlockId, usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Block {
    /// The inclusive start index of this block
    pub start: usize,
    /// The exclusive end index of this block
    pub end: usize,
    /// The optional next block of this level
    pub next: Option<BlockId>,
    /// The type of this block
    pub block_type: BlockType,
}

impl Block {
    #[inline]
    pub fn internal(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            next: None,
            block_type: BlockType::Internal(Vec::new()),
        }
    }
    
    #[inline]
    pub fn replace(&mut self, source: BlockId, offset: usize) {
        self.block_type = BlockType::Back(source, offset);
    }
    
    #[inline]
    pub fn set_next(&mut self, next: BlockId) {
        self.next = Some(next);
    }

    #[inline]
    pub fn is_back_block(&self) -> bool {
        matches!(self.block_type, BlockType::Back(_, _))
    }

    #[inline]
    pub fn is_internal(&self) -> bool {
        matches!(self.block_type, BlockType::Internal(_))
    }

    #[inline]
    pub fn is_adjacent(&self, next: &Block) -> bool {
        self.end == next.start
    }

    pub fn add_child(&mut self, block: BlockId) {
        match self.block_type {
            BlockType::Internal(ref mut children) => children.push(block),
            _ => panic!("attempted to add child to back block")
        }
    }

    /// The number of characters inside this block.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
