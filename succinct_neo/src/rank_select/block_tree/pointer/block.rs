#![allow(unused)]

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use id_arena::{Arena, Id};

use super::PointerBlockTree;

pub(crate) type BlockId = Id<Block>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BlockType {
    Internal {
        children: Vec<BlockId>,
        incident_pointers: u32,
    },
    Back,
}

#[derive(Debug, Clone)]
pub(crate) struct Block {
    /// The inclusive start index of this block
    pub start: usize,
    /// The exclusive end index of this block
    pub end: usize,
    /// The optional next block of this level
    pub next: Option<BlockId>,
    /// The type of this block
    pub block_type: BlockType,
    pub source: Option<BlockId>,
    pub offset: Option<usize>,
}

impl Block {
    #[inline]
    pub fn internal(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
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
    pub fn replace(&mut self, source: BlockId, offset: usize) {
        self.source = Some(source);
        self.offset = Some(offset);
        self.block_type = BlockType::Back;
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
    fn has_children(&self) -> bool {
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

    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn prune(blocks: &mut Arena<Block>, self_id: BlockId) {
        // SAFETY: we decouple this block's lifetime from the Arena in order to pass the arena to
        //  the recursive call
        //  This is safe, since a block cannot have itself as a child and the recursive calls only
        //  operate on this block's children
        let me = unsafe { (&mut blocks[self_id] as *mut Block).as_mut().unwrap() };
        match me.block_type {
            // If we hit an internal block we prune all children first
            BlockType::Internal {
                ref mut children,
                incident_pointers,
            } => {
                for &child_id in children.iter().rev() {
                    Self::prune(blocks, child_id);
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
                        blocks[block].start + offset + blocks[block].len(),
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
                    .map(|&child_id| &blocks[child_id])
                    .any(|child| child.has_children())
                {
                    return;
                }
                // We need to decrement all counters for the blocks the children point to
                for &child_id in children.iter() {
                    // SAFETY: We only modify the blocks the child is pointing to (which
                    // cannot be the child itself)
                    let child = unsafe { (&mut blocks[child_id] as *mut Block).as_mut().unwrap() };
                    child.decrement_pointer_count(blocks);
                }

                // Now replace this block with a back pointer
                me.replace(source, offset);
            }
            // If we hit a back block we increment the counters of the blocks it is pointing to
            BlockType::Back { .. } => me.increment_pointer_count(blocks),
        };
    }

    /// Increments the pointer count of the block(s) this block is pointing to (if it is a back
    /// block).
    fn increment_pointer_count(&mut self, blocks: &mut Arena<Block>) {
        if let BlockType::Back = self.block_type {
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
    fn decrement_pointer_count(&mut self, blocks: &mut Arena<Block>) {
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
