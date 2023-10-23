#![allow(unused)]

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use id_arena::{Arena, Id};

use crate::int_vec::{DynamicIntVec, IntVector};

use super::PointerBlockTree;

pub(crate) type BlockId = Id<Block>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BlockType {
    ///
    Internal {
        children: Vec<BlockId>,
        incident_pointers: u32,
    },
    /// If this block points back at another node
    Back,
}

#[derive(Debug, Clone)]
pub(crate) struct Block {
    /// The inclusive start index of this block
    pub start: usize,
    /// The exclusive end index of this block
    pub end: usize,
    /// This block's index on the level it is on
    pub index: usize,
    /// The optional next block of this level
    pub next: Option<BlockId>,
    /// The type of this block
    pub block_type: BlockType,
    pub source: Option<BlockId>,
    pub offset: Option<usize>,
}

impl Block {
    #[inline]
    pub fn internal(start: usize, end: usize, index: usize) -> Self {
        Self {
            start,
            end,
            index,
            next: None,
            block_type: BlockType::Internal {
                children: Vec::new(),
                incident_pointers: 0,
            },
            source: None,
            offset: None,
        }
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
        matches!(self.block_type, BlockType::Back { .. })
    }

    #[inline]
    pub fn is_internal(&self) -> bool {
        matches!(self.block_type, BlockType::Internal { .. })
    }

    #[inline]
    pub fn is_adjacent(&self, next: &Block) -> bool {
        self.end == next.start
    }

    #[inline]
    pub fn add_child(&mut self, block: BlockId) {
        match self.block_type {
            BlockType::Internal {
                ref mut children, ..
            } => children.push(block),
            _ => panic!("attempted to add child to back block"),
        }
    }

    pub fn get(&self, bt: &PointerBlockTree, i: usize) -> u8 {
        match self.block_type {
            BlockType::Internal { ref children, .. } => {
                if children.is_empty() {
                    return bt.input[self.start + i];
                }

                // SAFETY: We just checked that we have children
                let child_len = unsafe { bt.blocks[*children.get_unchecked(0)].len() };
                let child_idx = i / child_len;
                let new_i = i % child_len;
                bt.blocks[children[child_idx]].get(bt, new_i)
            }
            BlockType::Back => {
                let offset = unsafe { self.offset.unwrap_unchecked() };
                let source = unsafe { self.source.unwrap_unchecked() };
                if offset + i < self.len() {
                    bt.blocks[source].get(bt, offset + i)
                } else {
                    let source_id = bt.blocks[source].next.unwrap();
                    bt.blocks[source_id].get(bt, offset + i - self.len())
                }
            }
        }
    }

    #[inline]
    pub(super) fn has_children(&self) -> bool {
        match self.block_type {
            BlockType::Internal { ref children, .. } => !children.is_empty(),
            _ => false,
        }
    }

    /// The number of characters inside this block.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Increments the pointer count of the block(s) this block is pointing to (if it is a back
    /// block).
    pub(super) fn increment_pointer_count(&mut self, blocks: &mut Arena<Block>) {
        if let BlockType::Back { .. } = self.block_type {
            let offset = unsafe { self.offset.unwrap_unchecked() };
            let source = unsafe { self.source.unwrap_unchecked() };
            let source_block = &mut blocks[source];
            if let BlockType::Internal {
                ref mut incident_pointers,
                ..
            } = source_block.block_type
            {
                *incident_pointers += 1;
            }
            match source_block.next.map(|next| &mut blocks[next].block_type) {
                Some(BlockType::Internal {
                    ref mut incident_pointers,
                    ..
                }) if offset > 0 => {
                    *incident_pointers += 1;
                }
                _ => {}
            }
        }
    }

    /// Decrements the pointer count of the block(s) this block is pointing to (if it is a back
    /// block).
    pub(super) fn decrement_pointer_count(&mut self, blocks: &mut Arena<Block>) {
        if let BlockType::Back = self.block_type {
            let offset = unsafe { self.offset.unwrap_unchecked() };
            let source = unsafe { self.source.unwrap_unchecked() };
            let source_block = &mut blocks[source];
            if let BlockType::Internal {
                ref mut incident_pointers,
                ..
            } = source_block.block_type
            {
                *incident_pointers -= 1;
            }
            match source_block.next.map(|next| &mut blocks[next].block_type) {
                Some(BlockType::Internal {
                    ref mut incident_pointers,
                    ..
                }) if offset > 0 => {
                    *incident_pointers -= 1;
                }
                _ => {}
            }
        }
    }
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hashing only start and end should uniquely identify a block
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        // start and end should uniquely identify a block inside a block tree
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Block {}
