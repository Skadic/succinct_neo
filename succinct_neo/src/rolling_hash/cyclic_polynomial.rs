use rand::{rngs::SmallRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};

use super::{HashedBytes, RollingHash};

/// Cyclic polynomial rolling hashes for strings (or byte arrays)
///
/// # Examples
///
/// ```
/// use succinct_neo::rolling_hash::{CyclicPolynomial, RollingHash};
///
/// let s = "hashhash";
///
/// // Create a new cyclic polynomial hasher with a window size of 4.
/// let mut cc = CyclicPolynomial::new(s, 4);
///
/// let hash_0 = cc.hashed_bytes();
///
/// // Move forward 4 steps
/// cc.advance_n(4);
///
/// let hash_4 = cc.hashed_bytes();
///
/// // The hashes at indices 0 and 4 should be the same!
/// assert_eq!(hash_0, hash_4);
/// ```
pub struct CyclicPolynomial<'a> {
    /// The string we are hashing windows of
    s: &'a [u8],
    /// A table mapping a char to a unique value (also in 0..256)
    char_table: [u64; 256],
    /// The current offset into the string. We are hashing s[offset..offset + window_size]
    offset: usize,
    /// The size of the hashed window
    window_size: usize,
    /// The current hash value
    hash: u64,
    /// Seed for the random generation of the char table.
    /// This can be used if you want to create another hasher with the same table.
    seed: u64,
    // Whether we're at the end of the string.
    done: bool,
}

impl<'a> CyclicPolynomial<'a> {
    #[inline]
    /// Create a new cyclic polynomial hasher with a random seed.
    ///
    /// # Arguments
    ///
    /// * `s` - A reference to the string to hash.
    /// * `window_size` - The size of the window to hash at a time.
    pub fn new<T: AsRef<[u8]> + ?Sized>(s: &'a T, window_size: usize) -> Self {
        Self::with_seed(s, window_size, thread_rng().gen())
    }

    /// Create a new cyclic polynomial hasher with a given seed and table.
    /// This is for when you want to create a new hasher without needing to recompute the table.
    /// Note that this means that the given seed should be the seed that produces the char_table.
    ///
    /// # Arguments
    ///
    /// * `s` - A reference to the string to hash.
    /// * `window_size` - The size of the window to hash at a time.
    /// * `seed` - Seed for the random generation of the char table. This should be the seed that
    /// generated `char_table`
    /// * `char_table` - The `char_table` to use for this hasher. This should be the table created
    /// from `seed`.
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{CyclicPolynomial, RollingHash};
    ///
    /// let s = "hashhash";
    ///
    /// // Create a new cyclic polynomial hasher with a window size of 4;
    /// let mut cc = CyclicPolynomial::new(s, 4);
    ///
    /// /* ... do something ... */
    ///
    /// let seed = cc.seed();
    /// let char_table = *cc.char_table();
    ///
    /// // Different window sized are okay!
    /// // Now we have a new hasher without having to recompute the table
    /// let cc = CyclicPolynomial::with_table(s, 6, seed, &char_table);
    /// ```
    pub fn with_table<T: AsRef<[u8]> + ?Sized>(
        s: &'a T,
        window_size: usize,
        seed: u64,
        char_table: &[u64; 256],
    ) -> Self {
        let s = s.as_ref();
        let mut hash = 0;

        // Create initial hash value
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

    /// Creates a new hasher with a given seed which is used in the random generation of the
    /// `char_table`.
    ///
    /// # Arguments
    ///
    /// * `s` - A reference to the string to hash.
    /// * `window_size` - The size of the window to hash at a time.
    /// * `seed` - Seed for the random generation of the char table.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{CyclicPolynomial, RollingHash};
    ///
    /// let s = "hashhash";
    ///
    /// // Create a new cyclic polynomial hasher with a window size of 4 with a given seed.
    /// let mut cc = CyclicPolynomial::with_seed(s, 4, 12345);
    /// ```
    pub fn with_seed<T: AsRef<[u8]> + ?Sized>(s: &'a T, window_size: usize, seed: u64) -> Self {
        // Generate random character hash
        let mut char_table = [0; 256];
        for (i, c) in char_table.iter_mut().enumerate() {
            *c = i as u64;
        }
        let mut rng = SmallRng::seed_from_u64(seed);
        char_table.as_mut_slice().shuffle(&mut rng);

        Self::with_table(s, window_size, seed, &char_table)
    }

    /// Returns the seed that was used for the generation of this hasher's `char_table`.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{CyclicPolynomial, RollingHash};
    ///
    /// let s = "hashhash";
    /// let cc = CyclicPolynomial::with_seed(s, 4, 12345);
    ///
    /// assert_eq!(cc.seed(), 12345);
    /// ```
    #[inline]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the `char_table` used in this hasher.
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
        let outchar = self.s.get(self.offset).copied().unwrap_or_default() as usize;
        let inchar = self
            .s
            .get(self.offset + self.window_size)
            .copied()
            .unwrap_or_default() as usize;

        self.hash = self.hash.rotate_left(1)
            ^ self.char_table[outchar].rotate_left(self.window_size as u32)
            ^ self.char_table[inchar];

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
        if self.offset + self.window_size > self.s.len() {
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
        cc.advance_n(5);
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

        let num_distinct = string_source
            .as_bytes()
            .windows(window_size)
            .unique()
            .count();

        let mut map = HashedByteMap::<usize>::default();
        let cc = CyclicPolynomial::new(&string_source, window_size);
        let seed = cc.seed();

        for (i, s) in cc.enumerate() {
            map.insert(s, i);
        }

        assert_eq!(
            num_distinct,
            map.len(),
            "incorrect number of elements in map"
        );

        let cc = CyclicPolynomial::with_seed(&string_source, window_size, seed);
        for s in cc {
            assert!(map.contains_key(&s), "hashed value not found");
        }
    }
}
