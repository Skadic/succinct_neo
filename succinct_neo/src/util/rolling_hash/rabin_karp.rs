use super::{HashedBytes, RollingHash};

/// Rabin Karp rolling hashes for strings (or byte arrays)
pub struct RabinKarp<'a> {
    s: &'a [u8],
    offset: usize,
    window_size: usize,
    prime: u64,
    hash: u64,
    done: bool,
}

impl<'a> RabinKarp<'a> {
    pub fn new<T: AsRef<[u8]>>(s: &'a T, window_size: usize, prime: u64) -> Self {
        let s = s.as_ref();
        assert!(
            s.len() >= window_size,
            "string cannot be shorter than window size"
        );

        let mut hash = 0;
        for i in (1..window_size).rev() {
            hash = (hash + s[i] as u64) << 1;
            if hash.leading_zeros() < 2 {
                hash %= prime;
            }
        }
        hash += s[0] as u64;
        hash %= prime;

        Self {
            s,
            offset: 0,
            window_size,
            prime,
            hash,
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
        self.hash += self.prime;
        self.hash -= (self.s[self.offset] as u64) << (self.window_size - 1);
        self.hash <<= 1;
        self.hash += self.s[self.offset + self.window_size] as u64;
        self.hash %= self.prime;

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

    use crate::util::rolling_hash::HashedBytes;

    use super::RabinKarp;

    #[test]
    fn hash_test() {
        let string_source = "helloyouthere";
        let mut map = HashMap::<HashedBytes<'static>, usize>::new();
        let rk = RabinKarp::new(&string_source, 5, 7919);

        for (i, s) in rk.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(9, map.len());

        let rk = RabinKarp::new(&string_source, 5, 7919);
        for (i, s) in rk.enumerate() {
            assert_eq!(Some(&i), map.get(&s));
        }
    }
}
