use rand::{rngs::SmallRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};

use super::{HashedBytes, RollingHash};

/// Cyclic polynomial rolling hashes for strings (or byte arrays)
pub struct CyclicPolynomial<'a> {
    s: &'a [u8],
    char_table: [u64; 256],
    offset: usize,
    window_size: usize,
    hash: u64,
    seed: u64,
    done: bool,
}

impl<'a> CyclicPolynomial<'a> {
    #[inline]
    pub fn new<T: AsRef<[u8]>>(s: &'a T, window_size: usize) -> Self {
        Self::with_seed(s, window_size, thread_rng().gen())
    }

    pub fn with_table<T: AsRef<[u8]>>(
        s: &'a T,
        window_size: usize,
        seed: u64,
        char_table: &[u64; 256],
    ) -> Self {
        let s = s.as_ref();
        let mut hash = 0;

        for i in 0..window_size {
            hash ^= char_table[s[i] as usize].rotate_left((window_size - i - 1) as u32);
        }

        Self {
            s,
            char_table: *char_table,
            offset: 0,
            window_size,
            hash,
            seed,
            done: false,
        }
    }

    pub fn with_seed<T: AsRef<[u8]>>(s: &'a T, window_size: usize, seed: u64) -> Self {
        // Generate random character hash
        let mut char_table = [0; 256];
        for (i, c) in char_table.iter_mut().enumerate() {
            *c = i as u64;
        }
        let mut rng = SmallRng::seed_from_u64(seed);
        char_table.as_mut_slice().shuffle(&mut rng);

        Self::with_table(s, window_size, seed, &char_table)
    }

    #[inline]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn char_table(&self) -> &[u64; 256] {
        &self.char_table
    }
}

impl<'a> RollingHash<'a> for CyclicPolynomial<'a> {
    #[inline]
    fn hash(&self) -> u64 {
        self.hash
    }

    fn advance(&mut self) -> u64 {
        let outpos = self.offset;
        let inpos = self.offset + self.window_size;
        if inpos >= self.s.len() || self.done {
            self.done = true;
            self.offset = self.offset.min(self.s.len() - self.window_size);
            return self.hash;
        }

        self.hash = self.hash.rotate_left(1)
            ^ self.char_table[self.s[outpos] as usize].rotate_left(self.window_size as u32)
            ^ self.char_table[self.s[inpos] as usize];

        self.offset += 1;
        self.hash
    }

    #[inline]
    fn hashed_bytes(&self) -> HashedBytes<'a> {
        HashedBytes::new(
            &self.s[self.offset..self.offset + self.window_size],
            self.hash,
        )
    }
}

impl<'a> Iterator for CyclicPolynomial<'a> {
    type Item = HashedBytes<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let hb = self.hashed_bytes();
        self.advance();
        Some(hb)
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::rolling_hash::{HashedByteMap, RollingHash};

    use super::CyclicPolynomial;

    #[test]
    fn hash_eq_test() {
        let string_source = "hellohello";
        let mut cc = CyclicPolynomial::new(&string_source, 5);
        let hash1 = cc.hashed_bytes();
        for _ in 0..5 {
            cc.advance();
        }
        let hash2 = cc.hashed_bytes();
        assert_eq!(hash1.bytes, hash2.bytes, "backing bytes not equal");
        assert_eq!(hash1.hash, hash2.hash, "hashes not equal");
        assert_eq!(hash1, hash2, "hash objects not equal");
    }

    #[test]
    fn short_hash_test() {
        let string_source = "helloyouthere";
        let mut map = HashedByteMap::<'static, usize>::default();
        let cc = CyclicPolynomial::new(&string_source, 5);
        let seed = cc.seed();

        for (i, s) in cc.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(9, map.len());

        let cc = CyclicPolynomial::with_seed(&string_source, 5, seed);
        for (i, s) in cc.enumerate() {
            assert_eq!(Some(&i), map.get(&s));
        }
    }

    #[test]
    fn long_hash_test() {
        let s = "helloyouthere";
        let mut string_source = String::new();
        let repetitions = 100;
        let window_size = 200;
        for _ in 0..repetitions {
            string_source.push_str(s);
        }

        let num_distinct = string_source.as_bytes().windows(window_size).unique().count();

        let mut map = HashedByteMap::<usize>::default();
        let cc = CyclicPolynomial::new(&string_source, window_size);
        let seed = cc.seed();

        for (i, s) in cc.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(num_distinct, map.len(), "incorrect number of elements in map");

        let cc = CyclicPolynomial::with_seed(&string_source, window_size, seed);
        for s in cc {
            assert!(map.contains_key(&s), "hashed value not found");
        }
    }
}
