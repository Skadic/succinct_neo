use rand::{thread_rng, Rng};

use super::{HashedBytes, RollingHash};

/// Rabin Karp rolling hashes for strings (or byte arrays)
pub struct RabinKarp<'a> {
    s: &'a [u8],
    offset: usize,
    window_size: usize,
    poly: u64,
    pow: u64,
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
        let poly = find_irreducible_polynomial61();
        let mut pow = 1;

        let mut hash = 0;
        for i in 0..window_size - 1 {
            let c = s[i] as u64;
            hash <<= 1;
            hash += c;
            hash %= poly;
            //hash = ((hash + c) << 1) % poly;
            if i > 0 {
                pow = (pow << 1) % poly;
            }
        }
        hash += s[window_size - 1] as u64;
        hash %= poly;

        Self {
            s,
            offset: 0,
            window_size,
            hash,
            poly,
            pow,
            done: false,
        }
    }
}

/// Find an irreducible polynomial of degree 61.
/// We use 61 since it is the largest prime smaller than 64.
fn find_irreducible_polynomial61() -> u64 {
    const MASK_61: u64 = (1 << 62) - 1;
    let mut rng = thread_rng();
    let mut gen = rng.gen::<u64>() & MASK_61;

    while !check_irreducible(gen) {
        gen = rng.gen::<u64>() & MASK_61;
    }

    gen
}

/// Check whether the given degree-61 binary polynomial complement is irreducible.
///
/// # Arguments
///
/// * `poly` - A degree-61 binary polynomial. The 61 least significant bits of the given number
/// make the coefficients.
const fn check_irreducible(poly: u64) -> bool {
    const MASK_61: u64 = (1 << 62) - 1;
    let complement = !poly & MASK_61;
    let gcd = gcd::binary_u64(poly, complement);
    gcd == 1
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
        //self.hash += self.poly;
        //self.hash -= v;
        //self.hash <<= 1;
        //self.hash += self.s[self.offset + self.window_size] as u64;
        //self.hash %= self.poly;
        let c1 = self.s[self.offset];
        let c2 = self.s[self.offset + self.window_size];
        self.hash += self.poly;
        self.hash <<= 1;
        self.hash %= self.poly;
        self.hash -= self.pow * c1 as u64;
        self.hash += c2 as u64;
        self.hash %= self.poly;

        //self.hash = ((self.hash << 1) % self.poly + c2 as u64 - (self.pow * c1 as u64) % self.poly + self.poly) % self.poly;

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
            assert_eq!(prev_hash.hash, hash.hash, "hashes not equal at {i}");
            assert_eq!(
                prev_hash.bytes, hash.bytes,
                "backing bytes not equal at {i}"
            );
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
        let mut prev_hash1;
        for i in 0..string_source.len() - 5 {
            rk.advance();
            let hash = rk.hashed_bytes();
            assert_eq!(prev_hash2.hash, hash.hash, "hashes not equal at {i}");
            assert_eq!(prev_hash2.bytes, hash.bytes, "bytes not equal at {i}");
            assert_eq!(prev_hash2, hash, "hash objects not equal at {i}");
            prev_hash1 = hash;
            prev_hash2 = prev_hash1;
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
