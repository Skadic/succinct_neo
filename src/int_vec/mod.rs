use crate::traits::BlockType;

#[derive(Debug)]
pub struct IntVec<Block: BlockType = usize> {
    data: Vec<Block>,
    width: usize,
    capacity: usize,
    size: usize,
}

impl<Block: BlockType> IntVec<Block> {
    #[inline]
    pub fn new(width: usize) -> Self {
        Self::with_capacity(width, 8)
    }

    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        let block_size = std::mem::size_of::<Block>() * 8;
        let num_blocks = (capacity * width) / block_size;

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width,
            capacity: num_blocks * block_size / width,
            size: 0,
        };

        temp.data.push(Block::from_usize(0).unwrap());
        temp
    }

    #[inline]
    const fn block_width() -> usize {
        std::mem::size_of::<Block>() * 8
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << self.width) - 1
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * self.width) % Self::block_width()
    }

    pub fn push(&mut self, v: usize) {
        let offset = self.current_offset();
        let mask = self.mask();
        if offset == 0 {
            *self.data.last_mut().unwrap() |= Block::from_usize(v & mask).unwrap();
            self.size += 1;
            return;
        }

        // If we're wrapping into the next block
        if offset + self.width >= Self::block_width() {
            let fitting_bits = Self::block_width() - offset;
            let fitting_mask = (1 << fitting_bits) - 1;
            let mask = (1 << self.width) - 1;
            *self.data.last_mut().unwrap() |= Block::from_usize((v & fitting_mask) << offset).unwrap();
            let hi = (v & mask) >> fitting_bits;
            self.data
                .push(Block::from_usize(hi).unwrap());
            self.capacity = self.data.capacity() * Self::block_width() / self.width;
            self.size += 1;
            return;
        }

        *self.data.last_mut().unwrap() |= Block::from_usize((v & mask) << offset).unwrap();
        self.size += 1;
    }

    pub fn get(&self, index: usize) -> usize {
        let index_block = (index * self.width) / Self::block_width();
        let index_offset = (index * self.width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + self.width >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            let remaining_bits = self.width - fitting_bits;
            let lo = self.data[index_block].to_usize().unwrap() >> index_offset;
            let mask = (1 << remaining_bits) - 1;
            let hi = self.data[index_block + 1].to_usize().unwrap() & mask;
            println!("lo: {lo:b}, hi: {hi:b}");
            return (hi << fitting_bits) | lo;
        }

        let mask = (1 << self.width) - 1;
        let res = (self.data[index_block].to_usize().unwrap() >> index_offset) & mask;

            println!("res: {res:b}");

        res
    }

    pub fn raw_data(&self) -> &[Block] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn iter(&self) -> Iter<Block> {
        Iter { i: 0, v: self }
    }
}

impl<Block: BlockType> IntoIterator for IntVec<Block> {
    type Item = usize;

    type IntoIter = IntoIter<Block>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

pub struct IntoIter<Block: BlockType = usize> {
    i: usize,
    v: IntVec<Block>,
}

impl<Block: BlockType> Iterator for IntoIter<Block> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

pub struct Iter<'a, Block: 'a + BlockType = usize> {
    i: usize,
    v: &'a IntVec<Block>,
}

impl<'a, Block: 'a + BlockType> Iterator for Iter<'a, Block> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

#[cfg(test)]
mod test {
    use super::IntVec;

    #[test]
    pub fn test_push() {
        let mut v = IntVec::<u8>::new(3);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);

        assert_eq!(1, v.get(0));
        assert_eq!(2, v.get(1));
        assert_eq!(3, v.get(2));
        assert_eq!(4, v.get(3));
    }

    #[test]
    pub fn get_test() {
        let mut v = IntVec::<u16>::new(12);
        let mut test_v = Vec::new();
        let mut i = 1;
        for _ in 0..10 {
            v.push(i);
            test_v.push(i);
            i = (i << 1) | 1;
        }

        for (i, actual) in test_v.into_iter().enumerate() {
            assert_eq!(v.get(i), actual);
        }
    }

    #[test]
    pub fn iter_test() {
        let mut v = IntVec::<u16>::new(12);
        let mut test_v = Vec::new();
        let mut i = 1;
        for _ in 0..10 {
            v.push(i);
            test_v.push(i);
            i = (i << 1) | 1;
        }

        for block in v.raw_data() {
            println!("{block:016b}")
        }

        for (expect, actual) in test_v.into_iter().zip(v) {
            assert_eq!(expect, actual);
        }
    }
}
