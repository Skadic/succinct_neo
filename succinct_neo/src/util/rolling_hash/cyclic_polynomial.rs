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
            hash ^= char_table[s[i] as usize].rotate_left(i as u32);
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
        if self.offset + self.window_size >= self.s.len() || self.done {
            self.done = true;
            self.offset = self.offset.min(self.s.len() - self.window_size);
            return self.hash;
        }

        self.hash = self.hash.rotate_left(1)
            ^ self.char_table[self.s[self.offset] as usize].rotate_left(self.window_size as u32)
            ^ self.char_table[self.s[self.offset + self.window_size] as usize];

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
    use std::collections::HashMap;

    use crate::util::rolling_hash::HashedBytes;

    use super::CyclicPolynomial;

    #[test]
    fn hash_test() {
        let string_source = "helloyouthere";
        let mut map = HashMap::<HashedBytes<'static>, usize>::new();
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
}
