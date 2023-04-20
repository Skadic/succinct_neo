#![allow(unused)]

use id_arena::Id;

use super::PointerBlockTree;

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
    pub fn clear_next(&mut self) {
        self.next = None;
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

    #[inline]
    pub fn add_child(&mut self, block: BlockId) {
        match self.block_type {
            BlockType::Internal(ref mut children) => children.push(block),
            _ => panic!("attempted to add child to back block"),
        }
    }

    pub fn get(&self, bt: &PointerBlockTree, i: usize) -> u8 {
        match self.block_type {
            BlockType::Internal(ref children) => {
                if children.is_empty() {
                    return bt.input[self.start + i];
                }

                // SAFETY: We just checked that we have children
                let child_len = unsafe { bt.blocks[*children.get_unchecked(0)].len() };
                let child_idx = i / child_len;
                let new_i = i % child_len;
                bt.blocks[children[child_idx]].get(bt, new_i)
            }
            BlockType::Back(block_id, offset) if offset + i < self.len() => {
                bt.blocks[block_id].get(bt, offset + i)
            }
            BlockType::Back(block_id, offset) => {
                let source_id = bt.blocks[block_id].next.unwrap();
                bt.blocks[source_id].get(bt, offset + i - self.len())
            }
        }
    }

    /// The number of characters inside this block.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
