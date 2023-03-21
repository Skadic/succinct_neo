#[derive(Debug)]
struct BlockTree {
    input_length: usize,
    arity: usize,
    leaf_length: usize,
    /// sizes of a block for each level. index 0 = deepest
    level_block_sizes: Vec<usize>,
    /// num blocks for each level
    level_block_count: Vec<usize>,
}

impl BlockTree {
    pub fn new<T: AsRef<[u8]>>(input: T, arity: usize, leaf_length: usize) -> Self {
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

        println!("sizes: {:?}", self.level_block_sizes);
        println!("lengt: {:?}", self.level_block_count);
    }
}
#[cfg(test)]
mod test {
    use super::BlockTree;

    #[test]
    fn new_test() {
        let s = "verygoodverybaadverygoodverygood";
        println!("string len: {}", s.len());
        let bt = BlockTree::new(s, 2, 4);
    }
}
