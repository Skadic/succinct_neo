use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

use super::{HashedBytes, RollingHash};

const BASE: u64 = 257;
const PRIME: u64 = 8589935681;

/// Rabin Karp rolling hashes for strings (or byte arrays)
pub struct RabinKarp<'a> {
    s: &'a [u8],
    offset: usize,
    window_size: usize,
    /// When we need to remove a char from the hash we would actually need to multiply it by BASE^k and
    /// then subtract it. However since our hash is in the finite field GF(p),
    rem: u64,
    hash: u64,
    done: bool,
}

impl<'a> RabinKarp<'a> {
    pub fn new<T: AsRef<[u8]>>(s: &'a T, window_size: usize) -> Self {
        let s = s.as_ref();
        assert!(
            s.len() >= window_size,
            "string cannot be shorter than window size"
        );
        let mut rem = 1;

        let mut hash = 0;
        for c in s[0..window_size].iter().map(|&c| c as u64) {
            hash *= BASE;
            hash += c;
            hash %= PRIME;
        }

        for _ in 0..window_size - 1 {
            rem = (rem * BASE) % PRIME;
        }

        Self {
            s,
            offset: 0,
            window_size,
            hash,
            rem,
            done: false,
        }
    }
}

impl<'a> RollingHash<'a> for RabinKarp<'a> {
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
        let c_out = self.s[self.offset] as u64;
        let c_in = self.s[self.offset + self.window_size] as u64;

        self.hash += PRIME;
        self.hash -= (self.rem * c_out) % PRIME;
        //self.hash %= PRIME;
        self.hash *= BASE;
        self.hash += c_in;
        self.hash %= PRIME;

        self.offset += 1;
        self.hash()
    }

    #[inline]
    fn hashed_bytes(&self) -> HashedBytes<'a> {
        HashedBytes::new(
            &self.s[self.offset..self.offset + self.window_size],
            self.hash(),
        )
    }
}

impl<'a> Iterator for RabinKarp<'a> {
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

    use itertools::Itertools;

    use crate::util::rolling_hash::{HashedByteMap, HashedBytes, RollingHash};

    use super::RabinKarp;

    #[test]
    fn long_hash_test() {
        let s = "helloyouthere";
        let mut string_source = String::new();
        let repetitions = 100;
        let window_size = 200;
        for _ in 0..repetitions {
            string_source.push_str(s);
        }

        let num_distinct = string_source
            .as_bytes()
            .windows(window_size)
            .unique()
            .count();

        let mut map = HashedByteMap::<usize>::default();
        let cc = RabinKarp::new(&string_source, window_size);

        for (i, s) in cc.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(
            num_distinct,
            map.len(),
            "incorrect number of elements in map"
        );

        let cc = RabinKarp::new(&string_source, window_size);
        for s in cc {
            assert!(map.contains_key(&s), "hashed value not found");
        }
    }

    #[test]
    fn hash_eq_test() {
        let string_source = "hellohello";
        let mut rk = RabinKarp::new(&string_source, 5);
        let hash1 = rk.hashed_bytes();
        for _ in 0..5 {
            rk.advance();
        }
        let hash2 = rk.hashed_bytes();
        assert_eq!(hash1.bytes, hash2.bytes, "backing bytes not equal");
        assert_eq!(hash1.hash, hash2.hash, "hashes not equal");
        assert_eq!(hash1, hash2, "hash objects not equal");
    }

    #[test]
    fn same_hash_eq_test() {
        let string_source = "aaaaaaaaaaaaaaaaaaaaaaa";
        let mut rk = RabinKarp::new(&string_source, 5);
        let mut prev_hash = rk.hashed_bytes();
        for i in 0..string_source.len() - 5 {
            rk.advance();
            let hash = rk.hashed_bytes();
            assert_eq!(
                prev_hash.bytes, hash.bytes,
                "backing bytes not equal at {i}"
            );
            assert_eq!(prev_hash.hash, hash.hash, "hashes not equal at {i}");
            assert_eq!(prev_hash, hash, "hash objects not equal at {i}");
            prev_hash = hash;
        }
    }

    #[test]
    fn simple_hash_eq_test() {
        let string_source = "hahahahahahahahahahahahahahaha";
        let mut rk = RabinKarp::new(&string_source, 5);
        let mut prev_hash2 = rk.hashed_bytes();
        rk.advance();
        let mut prev_hash1 = rk.hashed_bytes();
        for i in 2..string_source.len() - 5 {
            rk.advance();
            let hash = dbg!(i, rk.hashed_bytes()).1;
            assert_eq!(prev_hash2.bytes, hash.bytes, "bytes not equal at {i}");
            assert_eq!(prev_hash2.hash, hash.hash, "hashes not equal at {i}");
            assert_eq!(prev_hash2, hash, "hash objects not equal at {i}");
            prev_hash2 = prev_hash1;
            prev_hash1 = hash;
        }
    }

    #[test]
    fn short_hash_test() {
        let string_source = "helloyouthere";
        let mut map = HashMap::<HashedBytes<'static>, usize>::new();
        let rk = RabinKarp::new(&string_source, 5);

        for (i, s) in rk.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(9, map.len());

        let rk = RabinKarp::new(&string_source, 5);
        for (i, s) in rk.enumerate() {
            assert_eq!(Some(&i), map.get(&s));
        }
    }
}
